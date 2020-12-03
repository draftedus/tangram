use backtrace::Backtrace;
use futures::FutureExt;
use num_traits::ToPrimitive;
use rusty_v8 as v8;
use serde_json::json;
use sha2::Digest;
use sourcemap::SourceMap;
use std::{
	borrow::Cow,
	cell::RefCell,
	collections::BTreeMap,
	convert::Infallible,
	fmt::Write,
	future::Future,
	panic::AssertUnwindSafe,
	path::{Path, PathBuf},
	rc::Rc,
	sync::Arc,
};
use tangram_util::{err, error::Result, zip};
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
			let hash = pinwheel::hash(&asset_path.to_str().unwrap());
			format!("/assets/{}.{}", hash, extension)
			}
		}};
}

#[macro_export]
macro_rules! client {
	() => {{
		let file_path = ::std::path::Path::new(file!());
		let client_crate_manifest_path = file_path.parent().unwrap().join("client/Cargo.toml");
		let hash = pinwheel::hash(client_crate_manifest_path.to_str().unwrap());
		format!("/js/{}.js", hash)
		}};
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
	pub fn dev(src_dir: PathBuf, dst_dir: PathBuf) -> Arc<Pinwheel> {
		let fs = RealFileSystem {
			dst_dir: dst_dir.clone(),
		};
		Arc::new(Pinwheel::Dev {
			src_dir,
			dst_dir,
			fs,
		})
	}

	pub fn prod(dir: include_dir::Dir<'static>) -> Arc<Pinwheel> {
		Arc::new(Pinwheel::Prod {
			fs: ProdFileSystem::Included(IncludedFileSystem { dir }),
		})
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

	pub fn render(&self, pagename: &str) -> Result<String> {
		self.render_with_props(pagename, json!({}))
	}

	pub fn render_with_props<T>(&self, pagename: &str, props: T) -> Result<String>
	where
		T: serde::Serialize,
	{
		THREAD_LOCAL_ISOLATE.with(|isolate| {
			// Compute the page entry from the pagename.
			let page_entry = page_entry_for_pagename(pagename);

			// In dev mode, compile the page.
			if let Pinwheel::Dev {
				src_dir, dst_dir, ..
			} = self
			{
				// Build the requested page.
				build_js_pages(src_dir, dst_dir, &[&page_entry])?;
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
			let static_or_server_js_url = if self.fs().file_exists(&static_js_url) {
				static_js_url
			} else if self.fs().file_exists(&server_js_url) {
				server_js_url
			} else {
				return Err(err!("could not find page {}", pagename));
			};
			let client_js_url = Url::parse("dst:/")
				.unwrap()
				.join(&format!("{}/client.js", page_entry))
				.unwrap();
			let client_js_src = if self.fs().file_exists(&client_js_url) {
				Some(format!("/{}/client.js", page_entry))
			} else {
				None
			};
			let client_wasm_js_url = Url::parse("dst:/")
				.unwrap()
				.join(&format!("{}/client_wasm.js", page_entry))
				.unwrap();
			let client_wasm_js_src = if self.fs().file_exists(&client_wasm_js_url) {
				Some(format!("/{}/client_wasm.js", page_entry))
			} else {
				None
			};

			// Read the pinwheel manifest.
			let pinwheel_manifest = self
				.fs()
				.read(&Url::parse("dst:/pinwheel_manifest.json").unwrap())
				.unwrap();
			let pinwheel_manifest: PinwheelManifest =
				serde_json::from_slice(&pinwheel_manifest).unwrap();

			// Create the scope.
			let mut isolate = isolate.borrow_mut();
			// In dev, reset the state to clear the module cache.
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

			// Get the default export from the static or server module.
			let page_module_namespace =
				evaluate_module(&mut scope, self.fs(), static_or_server_js_url.clone())?;
			let default_literal = v8::String::new(&mut scope, "default").unwrap().into();
			let page_module_default_export = page_module_namespace
				.get(&mut scope, default_literal)
				.ok_or_else(|| {
					err!(
						"Failed to get default export from {}.",
						static_or_server_js_url
					)
				})?;
			if !page_module_default_export.is_function() {
				return Err(err!(
					"The default export from {} must be a function.",
					static_or_server_js_url
				));
			}
			let page_module_default_export_function: v8::Local<v8::Function> =
				unsafe { v8::Local::cast(page_module_default_export) };

			// Get the JS and CSS sources from the pinwheel manifest.
			let css_srcs = pinwheel_manifest
				.css_srcs_for_page_entry
				.get(&page_entry)
				.unwrap();
			let js_srcs = pinwheel_manifest
				.js_srcs_for_page_entry
				.get(&page_entry)
				.unwrap();

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
			let js_srcs_literal = v8::String::new(&mut scope, "jsSrcs").unwrap();
			let js_srcs_array = v8::Array::new(&mut scope, js_srcs.len().to_i32().unwrap());
			for (i, js_src) in js_srcs.iter().enumerate() {
				let i = v8::Number::new(&mut scope, i.to_f64().unwrap()).into();
				let js_src = v8::String::new(&mut scope, js_src).unwrap().into();
				js_srcs_array.set(&mut scope, i, js_src);
			}
			page_info.set(&mut scope, js_srcs_literal.into(), js_srcs_array.into());
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

	pub async fn serve(self: Arc<Self>, host: std::net::IpAddr, port: u16) -> hyper::Result<()> {
		self.serve_with_handler(host, port, (), |pinwheel, _, request| {
			pinwheel.handle(request)
		})
		.await
	}

	pub async fn serve_with_handler<C, H, F>(
		self: Arc<Self>,
		host: std::net::IpAddr,
		port: u16,
		request_handler_context: C,
		request_handler: H,
	) -> hyper::Result<()>
	where
		C: Send + Sync + 'static,
		H: Fn(Arc<Pinwheel>, Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
		F: Future<Output = http::Response<hyper::Body>> + Send,
	{
		// Create a task local that will store the panic message and backtrace if a panic occurs.
		tokio::task_local! {
			static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, Backtrace)>>;
		}
		async fn service<C, H, F>(
			pinwheel: Arc<Pinwheel>,
			request_handler: Arc<H>,
			request_handler_context: Arc<C>,
			request: http::Request<hyper::Body>,
		) -> Result<http::Response<hyper::Body>, Infallible>
		where
			C: Send + Sync + 'static,
			H: Fn(Arc<Pinwheel>, Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
			F: Future<Output = http::Response<hyper::Body>> + Send,
		{
			let method = request.method().clone();
			let path = request.uri().path_and_query().unwrap().path().to_owned();
			let result =
				AssertUnwindSafe(request_handler(pinwheel, request_handler_context, request))
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
		let pinwheel = self.clone();
		let request_handler_context = Arc::new(request_handler_context);
		let service = hyper::service::make_service_fn(|_| {
			let request_handler = request_handler.clone();
			let pinwheel = pinwheel.clone();
			let request_handler_context = request_handler_context.clone();
			async move {
				Ok::<_, Infallible>(hyper::service::service_fn(move |request| {
					let request_handler = request_handler.clone();
					let pinwheel = pinwheel.clone();
					let request_handler_context = request_handler_context.clone();
					PANIC_MESSAGE_AND_BACKTRACE.scope(RefCell::new(None), async move {
						service(pinwheel, request_handler, request_handler_context, request).await
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

	pub async fn handle(
		self: Arc<Pinwheel>,
		request: http::Request<hyper::Body>,
	) -> http::Response<hyper::Body> {
		let uri = request.uri();
		let path_and_query = uri.path_and_query().unwrap();
		let path = path_and_query.path();
		// Render static pages in dev.
		if let Pinwheel::Dev { src_dir, .. } = self.as_ref() {
			let page_entry = page_entry_for_pagename(path);
			let page_exists = js_sources_for_page_entry(src_dir, &page_entry).is_some();
			if page_exists {
				let html = self.render(path).unwrap();
				let response = http::Response::builder()
					.status(http::StatusCode::OK)
					.body(hyper::Body::from(html))
					.unwrap();
				return response;
			}
		}
		// Serve static files from the static directory in dev.
		if let Pinwheel::Dev { src_dir, .. } = self.as_ref() {
			let static_path = src_dir.join("static").join(path.strip_prefix('/').unwrap());
			let exists = std::fs::metadata(&static_path)
				.map(|metadata| metadata.is_file())
				.unwrap_or(false);
			if exists {
				let body = std::fs::read(&static_path).unwrap();
				let mut response = http::Response::builder();
				if let Some(content_type) = content_type(&static_path) {
					response = response.header(http::header::CONTENT_TYPE, content_type);
				}
				let response = response.body(hyper::Body::from(body)).unwrap();
				return response;
			}
		}
		// Serve assets from the src_dir in dev.
		if let Pinwheel::Dev { .. } = self.as_ref() {
			if let Some(path) = path.strip_prefix("/assets") {
				let asset_path = Path::new(path.strip_prefix('/').unwrap());
				let exists = std::fs::metadata(&asset_path)
					.map(|metadata| metadata.is_file())
					.unwrap_or(false);
				if exists {
					let body = std::fs::read(&asset_path).unwrap();
					let mut response = http::Response::builder();
					if let Some(content_type) = content_type(&asset_path) {
						response = response.header(http::header::CONTENT_TYPE, content_type);
					}
					let response = response.body(hyper::Body::from(body)).unwrap();
					return response;
				}
			}
		}
		// Serve from the dst_dir.
		let url = Url::parse(&format!("dst:{}", path)).unwrap();
		if self.fs().file_exists(&url) {
			let data = self.fs().read(&url).unwrap();
			let mut response = http::Response::builder();
			if let Some(content_type) = content_type(Path::new(path)) {
				response = response.header(http::header::CONTENT_TYPE, content_type);
			}
			let response = response.body(hyper::Body::from(data)).unwrap();
			return response;
		}
		// Otherwise, 404.
		http::Response::builder()
			.status(http::StatusCode::NOT_FOUND)
			.body(hyper::Body::from("not found"))
			.unwrap()
	}
}

/// Compute the page entry from the pagename.
fn page_entry_for_pagename(pagename: &str) -> String {
	let page_entry = if pagename.ends_with('/') {
		pagename.to_owned() + "index"
	} else {
		pagename.to_owned()
	};
	let page_entry = page_entry.strip_prefix('/').unwrap().to_owned();
	page_entry
}

struct JsSourcesForPageEntryOutput {
	static_source_path: Option<PathBuf>,
	server_source_path: Option<PathBuf>,
	client_source_path: Option<PathBuf>,
}

/// Find the JS sources for the page entry in the source directory.
fn js_sources_for_page_entry(
	src_dir: &Path,
	page_entry: &str,
) -> Option<JsSourcesForPageEntryOutput> {
	let static_source_path = src_dir.join("pages").join(&page_entry).join("static.tsx");
	let static_source_path = if static_source_path.exists() {
		Some(static_source_path)
	} else {
		None
	};
	let server_source_path = src_dir.join("pages").join(&page_entry).join("server.tsx");
	let server_source_path = if server_source_path.exists() {
		Some(server_source_path)
	} else {
		None
	};
	let client_source_path = src_dir.join("pages").join(&page_entry).join("client.tsx");
	let client_source_path = if client_source_path.exists() {
		Some(client_source_path)
	} else {
		None
	};
	if static_source_path.is_none() && server_source_path.is_none() {
		return None;
	}
	Some(JsSourcesForPageEntryOutput {
		static_source_path,
		server_source_path,
		client_source_path,
	})
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

// V8 isolates are not `Send`, so we store one for each thread.
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

fn evaluate_module<'s>(
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
	let _ = module.instantiate_module(&mut try_catch_scope, resolve_module_callback);
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

fn resolve_module_callback<'s>(
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
	fn file_exists(&self, url: &Url) -> bool;
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>>;
}

pub struct RealFileSystem {
	dst_dir: PathBuf,
}

impl FileSystem for RealFileSystem {
	fn file_exists(&self, url: &Url) -> bool {
		let path = self.dst_dir.join(url.path().strip_prefix('/').unwrap());
		std::fs::metadata(path)
			.map(|metadata| metadata.is_file())
			.unwrap_or(false)
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
	fn file_exists(&self, url: &Url) -> bool {
		self.dir
			.get_file(url.path().strip_prefix('/').unwrap())
			.is_some()
	}
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>> {
		self.dir
			.get_file(url.path().strip_prefix('/').unwrap())
			.map(|d| d.contents.into())
			.ok_or_else(|| err!("no file found at url {}", url))
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PinwheelManifest {
	css_srcs_for_page_entry: BTreeMap<String, Vec<String>>,
	js_srcs_for_page_entry: BTreeMap<String, Vec<String>>,
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
	// Build the js pages.
	let page_entries = &page_entries
		.iter()
		.map(|page_entry| page_entry.as_str())
		.collect::<Vec<_>>();
	build_js_pages(src_dir, dst_dir, page_entries).unwrap();
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
	// Render the static pages and write them to the dst_dir.
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

pub fn build_js_pages(src_dir: &Path, dst_dir: &Path, page_entries: &[&str]) -> Result<()> {
	let pinwheel_manifest_path = dst_dir.join("pinwheel_manifest.json");
	let esbuild_metafile_path = dst_dir.join("metafile.json");
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
		format!("--metafile={}", esbuild_metafile_path.display()),
		format!("--outdir={}", dst_dir.display()),
	];
	for page_entry in page_entries {
		let js_sources = js_sources_for_page_entry(src_dir, page_entry).ok_or_else(|| {
			err!(
				"Could not find static.tsx or server.tsx for page entry {}",
				page_entry
			)
		})?;
		if let Some(static_source_path) = js_sources.static_source_path {
			args.push(format!("{}", static_source_path.display()));
		}
		if let Some(server_source_path) = js_sources.server_source_path {
			args.push(format!("{}", server_source_path.display()));
		}
		if let Some(client_source_path) = js_sources.client_source_path {
			args.push(format!("{}", client_source_path.display()));
		}
	}
	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(err!("esbuild {}", status.to_string()));
	}
	// Construct the pinwheel manifest from the esbuild metafile.
	let esbuild_metafile = std::fs::read(&esbuild_metafile_path)?;
	let esbuild_metafile: serde_json::Value = serde_json::from_slice(&esbuild_metafile)?;
	let esbuild_metafile_outputs = esbuild_metafile
		.as_object()
		.unwrap()
		.get("outputs")
		.unwrap()
		.as_object()
		.unwrap();
	// Collect all the CSS sources.
	let mut css_srcs = Vec::new();
	for output_path in esbuild_metafile_outputs.keys() {
		let output_path = Path::new(output_path);
		let extension = output_path.extension().unwrap().to_str().unwrap();
		if extension == "css" {
			let css_src = output_path.strip_prefix(&dst_dir).unwrap().to_owned();
			css_srcs.push(format!("/{}", css_src.display()));
		}
	}
	let mut css_srcs_for_page_entry = BTreeMap::new();
	let mut js_srcs_for_page_entry = BTreeMap::new();
	for page_entry in page_entries {
		css_srcs_for_page_entry.insert(page_entry.to_string(), css_srcs.clone());
		js_srcs_for_page_entry.insert(page_entry.to_string(), vec![]);
		// let output_path = dst_dir.join(page_entry).join("server.js");
		// let esbuild_metafile_entry = esbuild_metafile_outputs
		// 	.get(output_path.to_str().unwrap())
		// 	.unwrap()
		// 	.as_object()
		// 	.unwrap();
		// let js_srcs = esbuild_metafile_entry
		// 	.get("imports")
		// 	.unwrap()
		// 	.as_array()
		// 	.unwrap()
		// 	.iter()
		// 	.map(|value| {
		// 		let js_path = value
		// 			.as_object()
		// 			.unwrap()
		// 			.get("path")
		// 			.unwrap()
		// 			.as_str()
		// 			.unwrap();
		// 		let js_path = Path::new(js_path);
		// 		let js_src = js_path.strip_prefix(&dst_dir).unwrap().to_owned();
		// 		let js_src = format!("/{}", js_src.display());
		// 		js_src
		// 	})
		// 	.collect();
		// js_srcs_for_page_entry.insert(page_entry.to_string(), js_srcs);
	}
	let pinwheel_manifest = PinwheelManifest {
		css_srcs_for_page_entry,
		js_srcs_for_page_entry,
	};
	let pinwheel_manifest = serde_json::to_vec(&pinwheel_manifest)?;
	std::fs::write(pinwheel_manifest_path, pinwheel_manifest)?;
	std::fs::remove_file(esbuild_metafile_path)?;
	Ok(())
}

pub fn build_client_crate(
	src_dir: &Path,
	client_crate_manifest_paths: &[PathBuf],
	cargo_wasm_dir: &Path,
	dev: bool,
	dst_dir: &Path,
) -> Result<()> {
	let output_wasm_dir = dst_dir.join("js");
	let client_crate_package_names = client_crate_manifest_paths
		.iter()
		.map(|client_crate_manifest_path| {
			let client_crate_manifest =
				std::fs::read_to_string(&src_dir.join(client_crate_manifest_path))?;
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
				.unwrap()
				.to_owned();
			Ok(client_crate_name)
		})
		.collect::<Result<Vec<_>>>()?;
	let cmd = which("cargo")?;
	let mut args = vec![
		"build".to_owned(),
		"--target".to_owned(),
		"wasm32-unknown-unknown".to_owned(),
		"--target-dir".to_owned(),
		cargo_wasm_dir.to_str().unwrap().to_owned(),
	];
	if !dev {
		args.push("--release".to_owned())
	}
	for client_crate_package_name in client_crate_package_names.iter() {
		args.push("--package".to_owned());
		args.push(client_crate_package_name.clone());
	}
	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(err!("cargo {}", status.to_string()));
	}
	for (client_crate_manifest_path, client_crate_package_name) in zip!(
		client_crate_manifest_paths.iter(),
		client_crate_package_names.iter()
	) {
		let input_wasm_path = format!(
			"{}/wasm32-unknown-unknown/{}/{}.wasm",
			cargo_wasm_dir.to_str().unwrap(),
			if dev { "debug" } else { "release" },
			client_crate_package_name,
		);
		let hash = hash(client_crate_manifest_path.to_str().unwrap());
		wasm_bindgen_cli_support::Bindgen::new()
			.web(true)
			.unwrap()
			.keep_debug(dev)
			.remove_producers_section(true)
			.remove_name_section(true)
			.input_path(input_wasm_path)
			.out_name(&hash)
			.generate(&output_wasm_dir)
			.map_err(|error| err!(error))?;
	}
	Ok(())
}

pub fn hash(s: &str) -> String {
	let mut hash = sha2::Sha256::new();
	hash.update(s);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}
