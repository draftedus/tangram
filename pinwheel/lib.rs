use anyhow::{format_err, Result};
use hyper::{header, Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use rusty_v8 as v8;
use sourcemap::SourceMap;
use std::fmt::Write;
use std::{borrow::Cow, cell::RefCell, path::Path, path::PathBuf, rc::Rc};
use url::Url;

pub struct Pinwheel {
	src_dir: Option<PathBuf>,
	dst_dir: Option<PathBuf>,
	fs: Box<dyn FileSystem>,
}

#[derive(Debug)]
pub struct NotFoundError;

impl std::fmt::Display for NotFoundError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Not Found")
	}
}

impl std::error::Error for NotFoundError {}

#[derive(Debug)]
pub struct JSError(String);

impl std::fmt::Display for JSError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::error::Error for JSError {}

impl Pinwheel {
	pub fn dev(src_dir: PathBuf, dst_dir: PathBuf) -> Pinwheel {
		Pinwheel {
			src_dir: Some(src_dir),
			dst_dir: Some(dst_dir.clone()),
			fs: Box::new(RealFileSystem { dst_dir }),
		}
	}

	pub fn prod(dir: include_dir::Dir<'static>) -> Pinwheel {
		Pinwheel {
			src_dir: None,
			dst_dir: None,
			fs: Box::new(IncludedFileSystem { dir }),
		}
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
		if self.src_dir.is_some() {
			esbuild_single_page(
				self.src_dir.as_ref().unwrap(),
				self.dst_dir.as_ref().unwrap(),
				page_entry.clone(),
			)?;
		}

		// Determine the output URLs.
		let page_prefix = if self.src_dir.is_some() {
			""
		} else {
			&page_entry
		};
		let static_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/static.js", page_prefix))
			.unwrap();
		let server_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/server.js", page_prefix))
			.unwrap();
		let page_js_url = if self.fs.exists(&static_js_url) {
			static_js_url
		} else if self.fs.exists(&server_js_url) {
			server_js_url
		} else {
			return Err(NotFoundError.into());
		};
		let client_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&format!("{}/client.js", page_prefix))
			.unwrap();
		let client_js_src = if self.fs.exists(&client_js_url) {
			if self.src_dir.is_some() {
				Some("/client.js".to_owned())
			} else {
				Some(format!("/{}/client.js", page_entry))
			}
		} else {
			None
		};

		THREAD_LOCAL_ISOLATE.with(|isolate| {
			let mut isolate = isolate.borrow_mut();
			if self.src_dir.is_some() {
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
			let page_module_namespace =
				run_module(&mut scope, self.fs.as_ref(), page_js_url.clone())?;
			let default_literal = v8::String::new(&mut scope, "default").unwrap().into();
			let page_module_default_export = page_module_namespace
				.get(&mut scope, default_literal)
				.ok_or_else(|| format_err!("failed to get default export from {}", page_js_url))?;
			if !page_module_default_export.is_function() {
				return Err(format_err!(
					"default export from page module must be a functiuon"
				));
			}
			let page_module_default_export_function: v8::Local<v8::Function> =
				unsafe { v8::Local::cast(page_module_default_export) };

			// create the page info object
			let page_info = v8::Object::new(&mut scope);
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
				return Err(format_err!("{}", exception_string));
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

	pub async fn handle(&self, request: Request<Body>) -> Result<Response<Body>> {
		let uri = request.uri();
		let path_and_query = uri.path_and_query().unwrap();
		let path = path_and_query.path();
		// Serve static files from pinwheel.
		let mut static_path = path.to_owned();
		if static_path.ends_with('/') {
			static_path.push_str("index.html");
		}
		let static_path = static_path.strip_prefix('/').unwrap();
		// Serve from the static directory in dev.
		if let Some(src_dir) = self.src_dir.as_ref() {
			let static_path = src_dir.join("static").join(static_path);
			if static_path.exists() {
				let body = std::fs::read(&static_path)?;
				let mut response = Response::builder();
				if let Some(content_type) = content_type(static_path.to_str().unwrap()) {
					response = response.header(header::CONTENT_TYPE, content_type);
				}
				let response = response.body(Body::from(body)).unwrap();
				return Ok(response);
			}
		}
		// Serve from the `dst_dir`.
		let url = Url::parse(&format!("dst:/{}", static_path)).unwrap();
		if self.fs.exists(&url) {
			let data = self.fs.read(&url)?;
			let mut response = Response::builder();
			if let Some(content_type) = content_type(static_path) {
				response = response.header("content-type", content_type);
			}
			let response = response.body(Body::from(data)).unwrap();
			return Ok(response);
		}
		let html = self.render(path)?;
		let response = Response::builder()
			.status(StatusCode::OK)
			.body(Body::from(html))
			.unwrap();
		Ok(response)
	}
}

fn content_type(path: &str) -> Option<&'static str> {
	if path.ends_with(".js") {
		Some("text/javascript")
	} else if path.ends_with(".svg") {
		Some("image/svg+xml")
	} else if path.ends_with(".css") {
		Some("text/css")
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
		return Err(format_err!("{}", exception_string));
	}
	drop(try_catch_scope);
	let mut try_catch_scope = v8::TryCatch::new(scope);
	let _ = module.evaluate(&mut try_catch_scope);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		let exception_string = exception_to_string(&mut scope, exception);
		return Err(format_err!("{}", exception_string));
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
		return Err(format_err!("{}", exception_string));
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

struct RealFileSystem {
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

struct IncludedFileSystem {
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
			.ok_or_else(|| format_err!("no file found at url {}", url))
	}
}

pub fn build(src_dir: &Path, dst_dir: &Path) -> Result<()> {
	// Collect all pages in the pages directory.
	let mut page_entries = <Vec<String>>::new();
	let mut static_page_entries = <Vec<String>>::new();
	let pages_dir = src_dir.join("pages");
	for path in walkdir::WalkDir::new(&pages_dir) {
		let path = path.unwrap();
		let path = path.path();
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
	esbuild_pages(src_dir, dst_dir, &page_entries).unwrap();
	// Copy static files.
	let static_dir = src_dir.join("static");
	for path in walkdir::WalkDir::new(&static_dir) {
		let path = path.unwrap();
		let path = path.path();
		if path.is_file() {
			let out_path = dst_dir.join(path.strip_prefix(&static_dir).unwrap());
			std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
			std::fs::copy(path, out_path).unwrap();
		}
	}
	// Statically render the pages.
	let pinwheel = Pinwheel {
		src_dir: None,
		dst_dir: None,
		fs: Box::new(RealFileSystem {
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

pub fn esbuild_single_page(src_dir: &Path, dst_dir: &Path, page_entry: String) -> Result<()> {
	esbuild_pages(src_dir, dst_dir, &[page_entry])
}

pub fn esbuild_pages(src_dir: &Path, dst_dir: &Path, page_entries: &[String]) -> Result<()> {
	// Remove the `dst_dir` if it exists and create it.
	if dst_dir.exists() {
		std::fs::remove_dir_all(&dst_dir).unwrap();
	}
	std::fs::create_dir_all(&dst_dir).unwrap();
	let manifest_path = dst_dir.join("manifest.json");
	let cmd = "npx";
	let mut args = vec![
		"esbuild".to_owned(),
		"--format=esm".to_owned(),
		"--minify".to_owned(),
		"--bundle".to_owned(),
		"--splitting".to_owned(),
		// format!("--outbase={}/pages", src_dir.display()),
		"--resolve-extensions=.js,.jsx,.ts,.tsx,.css,.git,.jpg,.png,.svg".to_owned(),
		"--public-path=/".to_owned(),
		"--loader:.gif=file".to_owned(),
		"--loader:.jpg=file".to_owned(),
		"--loader:.png=file".to_owned(),
		"--loader:.svg=dataurl".to_owned(),
		"--sourcemap".to_owned(),
		format!("--metafile={}", manifest_path.display()),
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
			return Err(NotFoundError.into());
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
	let mut process = std::process::Command::new(cmd)
		.stderr(std::process::Stdio::inherit())
		.args(&args)
		.spawn()
		.unwrap();
	let status = process.wait().unwrap();
	if !status.success() {
		return Err(format_err!("esbuild {}", status.to_string()));
	}
	let collect_css = |css_src_dir: &Path, output_file_name: &str| {
		let mut css = String::new();
		for path in walkdir::WalkDir::new(&css_src_dir) {
			let path = path.unwrap();
			let path = path.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path).unwrap());
			}
		}
		std::fs::write(dst_dir.join(output_file_name), css).unwrap();
	};
	collect_css(&src_dir.join("../app"), "app.css");
	collect_css(&src_dir.join("../charts"), "charts.css");
	collect_css(&src_dir.join("../ui"), "ui.css");
	collect_css(&src_dir.join("../www"), "www.css");
	Ok(())
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
