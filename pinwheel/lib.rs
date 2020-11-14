use backtrace::Backtrace;
use futures::FutureExt;
use num_traits::ToPrimitive;
use rusty_v8 as v8;
use sha2::Digest;
use sourcemap::SourceMap;
use std::{
	borrow::Cow,
	cell::RefCell,
	convert::Infallible,
	fmt::Write,
	future::Future,
	panic::AssertUnwindSafe,
	path::{Path, PathBuf},
	rc::Rc,
	sync::Arc,
};
use tangram_util::{err, error::Result};
use url::Url;
use which::which;

#[macro_export]
macro_rules! asset {
	($asset_relative_path:literal) => {{
		let file_path = ::std::path::Path::new(file!());
		let asset_path = file_path.parent().unwrap().join($asset_relative_path);
		if cfg!(debug_assertions) {
			format!("/assets/{}", asset_path.display())
		} else {
			let extension = asset_path.extension().map(|e| e.to_str().unwrap()).unwrap();
			let hash = ::pinwheel::hash(&asset_path.to_str().unwrap());
			format!("/assets/{}.{}", hash, extension)
			}
		}};
}

#[macro_export]
macro_rules! client {
	($client_crate_relative_path:literal) => {{
		let file_path = ::std::path::Path::new(file!());
		let client_crate_manifest_path = file_path
			.parent()
			.unwrap()
			.join($client_crate_relative_path);
		let hash = ::pinwheel::hash(client_crate_manifest_path.to_str().unwrap());
		format!("/js/{}.js", hash)
		}};
}

// Create a task local that will store the panic message and backtrace if a panic occurs.
tokio::task_local! {
	static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, Backtrace)>>;
}

pub async fn serve<C, H, F>(
	host: std::net::IpAddr,
	port: u16,
	request_handler: H,
	request_handler_context: C,
) -> hyper::Result<()>
where
	C: Send + Sync + 'static,
	H: Fn(Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
	F: Future<Output = http::Response<hyper::Body>> + Send,
{
	// Install a panic hook that will record the panic message and backtrace if a panic occurs.
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
			panic_message_and_backtrace.borrow_mut().replace(value);
		})
	}));
	// Wrap the request handler and context with Arc to allow sharing a reference to it with each task.
	let request_handler = Arc::new(request_handler);
	let request_handler_context = Arc::new(request_handler_context);
	let service = hyper::service::make_service_fn(|_| {
		let request_handler = request_handler.clone();
		let request_handler_context = request_handler_context.clone();
		async move {
			Ok::<_, Infallible>(hyper::service::service_fn(move |request| {
				let request_handler = request_handler.clone();
				let request_handler_context = request_handler_context.clone();
				PANIC_MESSAGE_AND_BACKTRACE.scope(RefCell::new(None), async move {
					service(request_handler, request_handler_context, request).await
				})
			}))
		}
	});
	let addr = std::net::SocketAddr::new(host, port);
	let server = hyper::Server::try_bind(&addr)?;
	eprintln!("🚀 serving on port {}", port);
	server.serve(service).await?;
	std::panic::set_hook(hook);
	Ok(())
}

async fn service<C, H, F>(
	request_handler: Arc<H>,
	request_handler_context: Arc<C>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>, Infallible>
where
	C: Send + Sync + 'static,
	H: Fn(Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
	F: Future<Output = http::Response<hyper::Body>> + Send,
{
	let method = request.method().clone();
	let path = request.uri().path_and_query().unwrap().path().to_owned();
	let result = AssertUnwindSafe(request_handler(request_handler_context, request))
		.catch_unwind()
		.await;
	let response = result.unwrap_or_else(|_| {
		eprintln!("{} {} 500", method, path);
		let body = PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
			let panic_message_and_backtrace = panic_message_and_backtrace.borrow();
			let (message, backtrace) = panic_message_and_backtrace.as_ref().unwrap();
			format!("{}\n{:?}", message, backtrace)
		});
		http::Response::builder()
			.status(http::StatusCode::INTERNAL_SERVER_ERROR)
			.body(hyper::Body::from(body))
			.unwrap()
	});
	Ok(response)
}

pub enum Pinwheel {
	Dev {
		src_dir: PathBuf,
		dst_dir: PathBuf,
		fs: RealFileSystem,
	},
	Prod {
		fs: ProdFileSystem,
	},
}

pub enum ProdFileSystem {
	Real(RealFileSystem),
	Included(IncludedFileSystem),
}

impl Pinwheel {
	pub fn dev(src_dir: PathBuf, dst_dir: PathBuf) -> Pinwheel {
		let fs = RealFileSystem {
			dst_dir: dst_dir.clone(),
		};
		Pinwheel::Dev {
			src_dir,
			dst_dir,
			fs,
		}
	}

	pub fn prod(dir: include_dir::Dir<'static>) -> Pinwheel {
		Pinwheel::Prod {
			fs: ProdFileSystem::Included(IncludedFileSystem { dir }),
		}
	}

	fn fs(&self) -> &dyn FileSystem {
		match self {
			Pinwheel::Dev { fs, .. } => fs,
			Pinwheel::Prod { fs, .. } => match fs {
				ProdFileSystem::Real(fs) => fs,
				ProdFileSystem::Included(fs) => fs,
			},
		}
	}

	pub fn compile(&self, pagename: &str) -> Result<()> {
		// Compute the page entry from the pagename.
		let page_entry = if pagename.ends_with('/') {
			pagename.to_owned() + "index"
		} else {
			pagename.to_owned()
		};
		let page_entry = page_entry.strip_prefix('/').unwrap().to_owned();
		// In dev mode, compile the page.
		if let Pinwheel::Dev {
			src_dir, dst_dir, ..
		} = self
		{
			build_js_pages(true, src_dir, dst_dir, &[&page_entry])?;
		}
		Ok(())
	}

	pub fn render(&self, pagename: &str) -> Result<String> {
		self.render_with(pagename, serde_json::Value::Object(Default::default()))
	}

	pub fn render_with<T>(&self, pagename: &str, props: T) -> Result<String>
	where
		T: serde::Serialize,
	{
		// Compute the page entry from the pagename.
		let page_entry = if pagename.ends_with('/') {
			pagename.to_owned() + "index"
		} else {
			pagename.to_owned()
		};
		let page_entry = page_entry.strip_prefix('/').unwrap().to_owned();

		// In dev mode, compile the page.
		if let Pinwheel::Dev {
			src_dir, dst_dir, ..
		} = self
		{
			build_js_pages(true, src_dir, dst_dir, &[&page_entry])?;
		}

		// Determine the output URLs.
		let static_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/static.js", page_entry))
			.unwrap();
		let server_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/server.js", page_entry))
			.unwrap();
		let page_js_url = if self.fs().exists(&static_js_url) {
			static_js_url
		} else if self.fs().exists(&server_js_url) {
			server_js_url
		} else {
			return Err(err!("could not find page {}", pagename));
		};
		let client_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/client.js", page_entry))
			.unwrap();
		let client_js_src = if self.fs().exists(&client_js_url) {
			Some(format!("/{}/client.js", page_entry))
		} else {
			None
		};
		let client_wasm_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/client_wasm.js", page_entry))
			.unwrap();
		let client_wasm_js_src = if self.fs().exists(&client_wasm_js_url) {
			Some(format!("/{}/client_wasm.js", page_entry))
		} else {
			None
		};

		// Read the manifest.
		let manifest = self.fs().read(&Url::parse("dst:/manifest.json").unwrap())?;
		let manifest: serde_json::Value = serde_json::from_slice(&manifest)?;

		THREAD_LOCAL_ISOLATE.with(|isolate| {
			let mut isolate = isolate.borrow_mut();
			if let Pinwheel::Dev { .. } = self {
				isolate.set_slot(Rc::new(RefCell::new(State::default())));
			}
			let mut scope = v8::HandleScope::new(&mut *isolate);
			let context = v8::Context::new(&mut scope);
			let mut scope = v8::ContextScope::new(&mut scope, context);

			// Create the console global.
			let console = v8::Object::new(&mut scope);
			let log_literal = v8::String::new(&mut scope, "log").unwrap();
			let log = v8::Function::new(&mut scope, console_log).unwrap();
			console.set(&mut scope, log_literal.into(), log.into());
			let console_literal = v8::String::new(&mut scope, "console").unwrap();
			context
				.global(&mut scope)
				.set(&mut scope, console_literal.into(), console.into());

			// Get the default export from page.
			let page_module_namespace = run_module(&mut scope, self.fs(), page_js_url.clone())?;
			let default_literal = v8::String::new(&mut scope, "default").unwrap().into();
			let page_module_default_export = page_module_namespace
				.get(&mut scope, default_literal)
				.ok_or_else(|| err!("failed to get default export from {}", page_js_url))?;
			if !page_module_default_export.is_function() {
				return Err(err!("default export from page module must be a functiuon"));
			}
			let page_module_default_export_function: v8::Local<v8::Function> =
				unsafe { v8::Local::cast(page_module_default_export) };

			// Get the CSS sources from the manifest.
			let css_srcs = manifest
				.get("outputs")
				.unwrap()
				.as_object()
				.unwrap()
				.keys()
				.filter(|output| output.ends_with(".css"))
				.collect::<Vec<_>>();

			// Create the page info object.
			let page_info = v8::Object::new(&mut scope);
			let css_srcs_literal = v8::String::new(&mut scope, "cssSrcs").unwrap();
			let css_srcs_array = v8::Array::new(&mut scope, css_srcs.len().to_i32().unwrap());
			for (i, css_src) in css_srcs.iter().enumerate() {
				let i = v8::Number::new(&mut scope, i.to_f64().unwrap()).into();
				let css_src = v8::String::new(&mut scope, css_src).unwrap().into();
				css_srcs_array.set(&mut scope, i, css_src);
			}
			page_info.set(&mut scope, css_srcs_literal.into(), css_srcs_array.into());
			let client_js_src_literal = v8::String::new(&mut scope, "clientJsSrc").unwrap();
			let client_js_src_string = if let Some(client_js_src) = client_js_src {
				v8::String::new(&mut scope, &client_js_src).unwrap().into()
			} else {
				v8::undefined(&mut scope).into()
			};
			page_info.set(
				&mut scope,
				client_js_src_literal.into(),
				client_js_src_string,
			);
			let client_wasm_js_src_literal =
				v8::String::new(&mut scope, "clientWasmJsSrc").unwrap();
			let client_wasm_js_src_string = if let Some(client_wasm_js_src) = client_wasm_js_src {
				v8::String::new(&mut scope, &client_wasm_js_src)
					.unwrap()
					.into()
			} else {
				v8::undefined(&mut scope).into()
			};
			page_info.set(
				&mut scope,
				client_wasm_js_src_literal.into(),
				client_wasm_js_src_string,
			);
			let page_info = page_info.into();

			// Send the props to v8.
			let json = serde_json::to_string(&props)?;
			let json = v8::String::new(&mut scope, &json).unwrap();
			let props = v8::json::parse(&mut scope, json).unwrap();

			// Render the page.
			let mut try_catch_scope = v8::TryCatch::new(&mut scope);
			let undefined = v8::undefined(&mut try_catch_scope).into();
			let html = page_module_default_export_function.call(
				&mut try_catch_scope,
				undefined,
				&[page_info, props],
			);
			if try_catch_scope.has_caught() {
				let exception = try_catch_scope.exception().unwrap();
				let mut scope = v8::HandleScope::new(&mut try_catch_scope);
				let exception_string = exception_to_string(&mut scope, exception);
				return Err(err!("{}", exception_string));
			}
			let html = html.unwrap();
			drop(try_catch_scope);
			let html = html
				.to_string(&mut scope)
				.unwrap()
				.to_rust_string_lossy(&mut scope);

			Ok(html)
		})
	}

	pub async fn handle(
		&self,
		request: http::Request<hyper::Body>,
	) -> Result<http::Response<hyper::Body>> {
		let uri = request.uri();
		let path_and_query = uri.path_and_query().unwrap();
		let path = path_and_query.path();
		// Serve static files from the static directory in dev.
		if let Pinwheel::Dev { src_dir, .. } = self {
			let static_path = src_dir.join("static").join(path.strip_prefix('/').unwrap());
			if static_path.exists() {
				let body = std::fs::read(&static_path)?;
				let mut response = http::Response::builder();
				if let Some(content_type) = content_type(&static_path) {
					response = response.header(http::header::CONTENT_TYPE, content_type);
				}
				let response = response.body(hyper::Body::from(body)).unwrap();
				return Ok(response);
			}
		}
		// Serve assets from the src_dir in dev.
		if let Pinwheel::Dev { .. } = self {
			if let Some(path) = path.strip_prefix("/assets") {
				let asset_path = Path::new(path.strip_prefix('/').unwrap());
				if asset_path.exists() {
					let body = std::fs::read(&asset_path)?;
					let mut response = http::Response::builder();
					if let Some(content_type) = content_type(&asset_path) {
						response = response.header(http::header::CONTENT_TYPE, content_type);
					}
					let response = response.body(hyper::Body::from(body)).unwrap();
					return Ok(response);
				}
			}
		}
		// Serve from the dst_dir.
		let url = Url::parse(&format!("dst:{}", path)).unwrap();
		if self.fs().exists(&url) {
			let data = self.fs().read(&url)?;
			let mut response = http::Response::builder();
			if let Some(content_type) = content_type(Path::new(path)) {
				response = response.header(http::header::CONTENT_TYPE, content_type);
			}
			let response = response.body(hyper::Body::from(data)).unwrap();
			return Ok(response);
		}
		// Render a page if one exists.
		let url = Url::parse(&format!("dst:{}", path)).unwrap();
		if self.fs().exists(&url) {
			let html = self.render(path)?;
			let response = http::Response::builder()
				.status(http::StatusCode::OK)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
		// Otherwise, 404.
		let response = http::Response::builder()
			.status(http::StatusCode::NOT_FOUND)
			.body(hyper::Body::from("not found"))
			.unwrap();
		Ok(response)
	}
}

fn content_type(path: &Path) -> Option<&'static str> {
	let path = path.to_str().unwrap();
	if path.ends_with(".css") {
		Some("text/css")
	} else if path.ends_with(".js") {
		Some("text/javascript")
	} else if path.ends_with(".svg") {
		Some("image/svg+xml")
	} else if path.ends_with(".wasm") {
		Some("application/wasm")
	} else {
		None
	}
}

thread_local!(static THREAD_LOCAL_ISOLATE: RefCell<v8::OwnedIsolate> = {
	static V8_INIT: std::sync::Once = std::sync::Once::new();
	V8_INIT.call_once(|| {
		let platform = v8::new_default_platform().unwrap();
		v8::V8::initialize_platform(platform);
		v8::V8::initialize();
	});
	let mut isolate = v8::Isolate::new(Default::default());
	isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
	isolate.set_slot(Rc::new(RefCell::new(State::default())));
	RefCell::new(isolate)
});

#[derive(Default)]
struct State {
	module_handles: Vec<ModuleHandle>,
}

pub struct ModuleHandle {
	id: i32,
	url: Url,
	module: v8::Global<v8::Module>,
	source_map: Option<SourceMap>,
}

fn get_state(isolate: &mut v8::Isolate) -> Rc<RefCell<State>> {
	isolate.get_slot::<Rc<RefCell<State>>>().unwrap().clone()
}

fn get_module_handle_with_id(state: &State, id: i32) -> Option<&ModuleHandle> {
	state
		.module_handles
		.iter()
		.find(|module_handle| module_handle.id == id)
}

fn get_module_handle_for_url<'a>(state: &'a State, url: &Url) -> Option<&'a ModuleHandle> {
	state
		.module_handles
		.iter()
		.find(|module_handle| &module_handle.url == url)
}

fn run_module<'s>(
	scope: &mut v8::HandleScope<'s>,
	fs: &dyn FileSystem,
	url: Url,
) -> Result<v8::Local<'s, v8::Object>> {
	let module_id = load_module(scope, fs, url)?;
	let state = get_state(scope);
	let state = state.borrow();
	let module = &get_module_handle_with_id(&state, module_id).unwrap().module;
	let module = v8::Local::new(scope, module);
	drop(state);
	let mut try_catch_scope = v8::TryCatch::new(scope);
	let _ = module.instantiate_module(&mut try_catch_scope, module_resolve_callback);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		let exception_string = exception_to_string(&mut scope, exception);
		return Err(err!("{}", exception_string));
	}
	drop(try_catch_scope);
	let mut try_catch_scope = v8::TryCatch::new(scope);
	let _ = module.evaluate(&mut try_catch_scope);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		let exception_string = exception_to_string(&mut scope, exception);
		return Err(err!("{}", exception_string));
	}
	drop(try_catch_scope);
	let namespace = module.get_module_namespace();
	let object = namespace.to_object(scope).unwrap();
	Ok(object)
}

/// Load a module at the specified path and return the module id.
fn load_module(scope: &mut v8::HandleScope, fs: &dyn FileSystem, url: Url) -> Result<i32> {
	// Return the id for an existing module if a module at the specified path has alread been loaded.
	let state = get_state(scope);
	let state = state.borrow();
	let existing_module = get_module_handle_for_url(&state, &url);
	if let Some(existing_module) = existing_module {
		return Ok(existing_module.id);
	}
	drop(state);

	// Define the origin.
	let resource_name = v8::String::new(scope, &url.to_string()).unwrap();
	let resource_line_offset = v8::Integer::new(scope, 0);
	let resource_column_offset = v8::Integer::new(scope, 0);
	let resource_is_shared_cross_origin = v8::Boolean::new(scope, false);
	let script_id = v8::Integer::new(scope, 1);
	let source_map_url = v8::undefined(scope).into();
	let resource_is_opaque = v8::Boolean::new(scope, true);
	let is_wasm = v8::Boolean::new(scope, false);
	let is_module = v8::Boolean::new(scope, true);
	let origin = v8::ScriptOrigin::new(
		resource_name.into(),
		resource_line_offset,
		resource_column_offset,
		resource_is_shared_cross_origin,
		script_id,
		source_map_url,
		resource_is_opaque,
		is_wasm,
		is_module,
	);

	// Read the source.
	let code = fs.read(&url)?;
	let code = std::str::from_utf8(code.as_ref())?;
	let source = v8::script_compiler::Source::new(v8::String::new(scope, code).unwrap(), &origin);

	// Read the source map.
	let source_map_url = match sourcemap::locate_sourcemap_reference_slice(code.as_bytes()).unwrap()
	{
		Some(source_map_reference) => Some(url.join(source_map_reference.get_url()).unwrap()),
		None => None,
	};
	let source_map = if let Some(source_map_url) = source_map_url {
		let source_map = fs.read(&source_map_url)?;
		let source_map = sourcemap::SourceMap::from_slice(source_map.as_ref())?;
		Some(source_map)
	} else {
		None
	};

	// Compile the module.
	let mut try_catch_scope = v8::TryCatch::new(scope);
	let module = v8::script_compiler::compile_module(&mut try_catch_scope, source);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		let exception_string = exception_to_string(&mut scope, exception);
		return Err(err!("{}", exception_string));
	}
	let module = module.unwrap();
	drop(try_catch_scope);

	// Register the module.
	let id = module.get_identity_hash();
	let state = get_state(scope);
	let mut state = state.borrow_mut();
	let global_module = <v8::Global<v8::Module>>::new(scope, module);
	state.module_handles.push(ModuleHandle {
		id,
		url,
		module: global_module,
		source_map,
	});
	drop(state);

	// Load each of the module's dependencies recursively.
	for i in 0..module.get_module_requests_length() {
		let module_request = module.get_module_request(i);
		let specifier = module_request.to_rust_string_lossy(scope);
		let state = get_state(scope);
		let state = state.borrow();
		let referrer_url = &get_module_handle_with_id(&state, id).unwrap().url;
		let url = referrer_url.join(&specifier).unwrap();
		drop(state);
		load_module(scope, fs, url)?;
	}

	Ok(id)
}

fn module_resolve_callback<'s>(
	context: v8::Local<'s, v8::Context>,
	specifier: v8::Local<'s, v8::String>,
	referrer: v8::Local<'s, v8::Module>,
) -> Option<v8::Local<'s, v8::Module>> {
	let mut scope = unsafe { v8::CallbackScope::new(context) };
	let specifier = specifier.to_rust_string_lossy(&mut scope);
	let id = referrer.get_identity_hash();
	let state = get_state(&mut scope);
	let state = state.borrow();
	let referrer_url = &get_module_handle_with_id(&state, id).unwrap().url;
	let url = referrer_url.join(&specifier).unwrap();
	let module = &get_module_handle_for_url(&state, &url).unwrap().module;
	Some(v8::Local::new(&mut scope, module))
}

fn console_log(
	scope: &mut v8::HandleScope,
	args: v8::FunctionCallbackArguments,
	_rv: v8::ReturnValue,
) {
	let mut result = String::new();
	for i in 0..args.length() {
		let arg = args.get(i);
		let arg_string = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
		if i > 0 {
			result.push(' ');
		}
		result.push_str(&arg_string);
	}
	println!("{}", result);
}

/// Render an exception to a string. The string will include the exception's message and a stack trace with source maps applied.
fn exception_to_string(scope: &mut v8::HandleScope, exception: v8::Local<v8::Value>) -> String {
	let mut string = String::new();
	let message = exception
		.to_string(scope)
		.unwrap()
		.to_rust_string_lossy(scope);
	writeln!(&mut string, "{}", message).unwrap();
	let stack_trace = v8::Exception::get_stack_trace(scope, exception).unwrap();
	for i in 0..stack_trace.get_frame_count() {
		let stack_trace_frame = stack_trace.get_frame(scope, i).unwrap();
		let source_url = Url::parse(
			&stack_trace_frame
				.get_script_name(scope)
				.unwrap()
				.to_rust_string_lossy(scope),
		)
		.unwrap();
		let source_line = stack_trace_frame.get_line_number();
		let source_column = stack_trace_frame.get_column();
		let state = get_state(scope);
		let state = state.borrow();
		let module_handle = get_module_handle_for_url(&state, &source_url).unwrap();
		let token = module_handle
			.source_map
			.as_ref()
			.unwrap()
			.lookup_token(
				(source_line - 1).to_u32().unwrap(),
				(source_column - 1).to_u32().unwrap(),
			)
			.unwrap();
		write!(
			&mut string,
			"{}:{}:{}",
			token.get_source().unwrap_or("<unknown>"),
			token.get_src_line() + 1,
			token.get_src_col() + 1,
		)
		.unwrap();
		if i < stack_trace.get_frame_count() - 1 {
			writeln!(&mut string).unwrap();
		}
	}
	string
}

trait FileSystem: Send + Sync {
	fn exists(&self, url: &Url) -> bool;
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>>;
}

pub struct RealFileSystem {
	dst_dir: PathBuf,
}

impl FileSystem for RealFileSystem {
	fn exists(&self, url: &Url) -> bool {
		self.dst_dir
			.join(url.path().strip_prefix('/').unwrap())
			.exists()
	}
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>> {
		std::fs::read(self.dst_dir.join(url.path().strip_prefix('/').unwrap()))
			.map(|d| d.into())
			.map_err(|e| e.into())
	}
}

pub struct IncludedFileSystem {
	dir: include_dir::Dir<'static>,
}

impl FileSystem for IncludedFileSystem {
	fn exists(&self, url: &Url) -> bool {
		self.dir.contains(url.path().strip_prefix('/').unwrap())
	}
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>> {
		self.dir
			.get_file(url.path().strip_prefix('/').unwrap())
			.map(|d| d.contents.into())
			.ok_or_else(|| err!("no file found at url {}", url))
	}
}

pub fn build(src_dir: &Path, dst_dir: &Path) -> Result<()> {
	// Collect all pages in the pages directory.
	let mut page_entries = <Vec<String>>::new();
	let mut static_page_entries = <Vec<String>>::new();
	let pages_dir = src_dir.join("pages");
	for entry in walkdir::WalkDir::new(&pages_dir) {
		let entry = entry.unwrap();
		let path = entry.path();
		match path.file_stem().unwrap().to_str().unwrap() {
			"static" | "server" => {
				let page_entry: String = path
					.strip_prefix(&pages_dir)
					.unwrap()
					.parent()
					.unwrap()
					.to_owned()
					.to_str()
					.unwrap()
					.to_owned();
				page_entries.push(page_entry.clone());
				if path.file_stem().unwrap().to_str().unwrap() == "static" {
					static_page_entries.push(page_entry);
				}
			}
			_ => {}
		}
	}
	// Build the pages.
	build_js_pages(
		false,
		src_dir,
		dst_dir,
		&page_entries
			.iter()
			.map(|page_entry| page_entry.as_str())
			.collect::<Vec<_>>(),
	)
	.unwrap();
	// Copy static files.
	let static_dir = src_dir.join("static");
	for entry in walkdir::WalkDir::new(&static_dir) {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_file() {
			let out_path = dst_dir.join(path.strip_prefix(&static_dir).unwrap());
			std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
			std::fs::copy(path, out_path).unwrap();
		}
	}
	// Statically render the pages.
	let pinwheel = Pinwheel::Prod {
		fs: ProdFileSystem::Real(RealFileSystem {
			dst_dir: dst_dir.to_owned(),
		}),
	};
	for page_entry in static_page_entries {
		let mut pagename = String::from("/") + &page_entry;
		if pagename.ends_with("/index") {
			pagename = pagename.strip_suffix("index").unwrap().to_owned();
		}
		let html = pinwheel.render(&pagename)?;
		let html_path = dst_dir.join(page_entry.to_owned() + ".html");
		let html_parent = html_path.parent().unwrap();
		std::fs::create_dir_all(html_parent).unwrap();
		std::fs::write(html_path, html).unwrap();
	}
	Ok(())
}

pub fn build_js_pages(
	_dev: bool,
	src_dir: &Path,
	dst_dir: &Path,
	page_entries: &[&str],
) -> Result<()> {
	let metafile_path = dst_dir.join("manifest.json");
	let cmd = which("npx").unwrap();
	let mut args = vec![
		"esbuild".to_owned(),
		"--format=esm".to_owned(),
		"--minify".to_owned(),
		"--bundle".to_owned(),
		"--splitting".to_owned(),
		format!("--outbase={}/pages", src_dir.display()),
		"--resolve-extensions=.js,.jsx,.ts,.tsx,.css,.gif,.jpg,.png,.svg,.woff2".to_owned(),
		"--public-path=/".to_owned(),
		"--loader:.gif=file".to_owned(),
		"--loader:.jpg=file".to_owned(),
		"--loader:.png=file".to_owned(),
		"--loader:.svg=dataurl".to_owned(),
		"--loader:.woff2=file".to_owned(),
		"--sourcemap".to_owned(),
		format!("--metafile={}", metafile_path.display()),
		format!("--outdir={}", dst_dir.display()),
	];
	for page_entry in page_entries {
		let static_source_path = src_dir.join("pages").join(&page_entry).join("static.tsx");
		let server_source_path = src_dir.join("pages").join(&page_entry).join("server.tsx");
		let client_source_path = src_dir.join("pages").join(&page_entry).join("client.tsx");
		let static_source_path_exists = static_source_path.exists();
		let server_source_path_exists = server_source_path.exists();
		let client_source_path_exists = client_source_path.exists();
		if !static_source_path_exists && !server_source_path_exists {
			return Err(err!(
				"could not find static.tsx or server.tsx for {}",
				page_entry
			));
		}
		if static_source_path_exists {
			args.push(format!("{}", static_source_path.display()));
		} else if server_source_path_exists {
			args.push(format!("{}", server_source_path.display()));
		}
		if client_source_path_exists {
			args.push(format!("{}", client_source_path.display()));
		}
	}
	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(err!("esbuild {}", status.to_string()));
	}
	// Strip the dst_dir prefix from the paths in the esbuild manifest.
	let metafile = std::fs::read(&metafile_path)?;
	let mut metafile: serde_json::Value = serde_json::from_slice(&metafile)?;
	let outputs = metafile
		.as_object()
		.unwrap()
		.get("outputs")
		.unwrap()
		.as_object()
		.unwrap();
	let mut new_outputs = serde_json::Map::new();
	for (key, value) in outputs.iter() {
		let key = key
			.strip_prefix(dst_dir.to_str().unwrap())
			.unwrap()
			.to_owned();
		new_outputs.insert(key, value.clone());
	}
	metafile
		.as_object_mut()
		.unwrap()
		.insert("outputs".to_owned(), new_outputs.into());
	let metafile = serde_json::to_vec(&metafile)?;
	std::fs::write(metafile_path, metafile)?;
	Ok(())
}

pub fn build_client_crate(
	src_dir: &Path,
	client_crate_manifest_path: &Path,
	cargo_wasm_dir: &Path,
	dev: bool,
	dst_dir: &Path,
) -> Result<String> {
	let client_crate_manifest = std::fs::read_to_string(&src_dir.join(client_crate_manifest_path))?;
	let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
	let client_crate_name = client_crate_manifest
		.as_table()
		.unwrap()
		.get("package")
		.unwrap()
		.as_table()
		.unwrap()
		.get("name")
		.unwrap()
		.as_str()
		.unwrap();
	let cmd = which("cargo")?;
	let mut args = vec![
		"build".to_owned(),
		"--target".to_owned(),
		"wasm32-unknown-unknown".to_owned(),
		"--target-dir".to_owned(),
		cargo_wasm_dir.to_str().unwrap().to_owned(),
		"--package".to_owned(),
		client_crate_name.to_owned(),
	];
	if !dev {
		args.push("--release".to_owned())
	}
	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(err!("cargo {}", status.to_string()));
	}
	let input_wasm_path = format!(
		"{}/wasm32-unknown-unknown/{}/{}.wasm",
		cargo_wasm_dir.to_str().unwrap(),
		if dev { "debug" } else { "release" },
		client_crate_name,
	);
	let hash = hash(client_crate_manifest_path.to_str().unwrap());
	let output_wasm_path = dst_dir.join("js");
	wasm_bindgen_cli_support::Bindgen::new()
		.web(true)
		.unwrap()
		.keep_debug(dev)
		.remove_producers_section(true)
		.remove_name_section(true)
		.input_path(input_wasm_path)
		.out_name(&hash)
		.generate(output_wasm_path)
		.map_err(|error| err!(error))?;
	Ok(hash)
}

pub fn hash(s: &str) -> String {
	let mut hash = sha2::Sha256::new();
	hash.update(s);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}
