use anyhow::{format_err, Result};
use hyper::{header, Body, Request, Response, StatusCode};
use rusty_v8 as v8;
use sourcemap::SourceMap;
use std::{borrow::Cow, cell::RefCell, path::Path, path::PathBuf, rc::Rc};
use url::Url;

pub struct Pinwheel {
	src_dir: Option<PathBuf>,
	dst_dir: Option<PathBuf>,
	fs: Box<dyn VirtualFileSystem>,
}

impl Pinwheel {
	pub fn dev(src_dir: PathBuf, dst_dir: PathBuf) -> Self {
		Self {
			src_dir: Some(src_dir),
			dst_dir: Some(dst_dir.clone()),
			fs: Box::new(RealFileSystem { dst_dir }),
		}
	}

	pub fn prod(dir: include_dir::Dir<'static>) -> Self {
		Self {
			src_dir: None,
			dst_dir: None,
			fs: Box::new(IncludedFileSystem { dir }),
		}
	}

	pub fn render<T>(&self, pagename: &str, props: T) -> Result<String>
	where
		T: serde::Serialize,
	{
		// compute the page entry from the pagename
		let page_entry = if pagename.ends_with('/') {
			pagename.to_string() + "index"
		} else {
			pagename.to_string()
		};
		let page_entry = page_entry.strip_prefix('/').unwrap();

		// in dev mode compile the page
		if self.src_dir.is_some() {
			esbuild_single_page(
				self.src_dir.as_ref().unwrap(),
				self.dst_dir.as_ref().unwrap(),
				page_entry,
			)
			.unwrap();
		}

		// determine output urls
		let document_js_url = Url::parse("dst:/document.js").unwrap();
		let page_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&("pages/".to_string() + page_entry + "/static.js"))
			.unwrap();
		let client_js_url = Url::parse("dst:/")
			.unwrap()
			.join(&("pages/".to_string() + page_entry + "/client.js"))
			.unwrap();
		let client_js_pathname = if self.fs.exists(&client_js_url) {
			Some(PathBuf::from("/pages").join(&page_entry).join("client.js"))
		} else {
			None
		};

		THREAD_LOCAL_ISOLATE.with(|isolate| {
			let mut isolate = isolate.borrow_mut();
			if self.src_dir.is_some() {
				let state = State {
					module_handles: Vec::new(),
				};
				isolate.set_slot(Rc::new(RefCell::new(state)));
			}
			let mut scope = v8::HandleScope::new(&mut *isolate);
			let context = v8::Context::new(&mut scope);
			let mut scope = v8::ContextScope::new(&mut scope, context);

			// create console global
			let console = v8::Object::new(&mut scope);
			let log_string = v8::String::new(&mut scope, "log").unwrap();
			let log = v8::Function::new(&mut scope, console_log).unwrap();
			console.set(&mut scope, log_string.into(), log.into());
			let console_string = v8::String::new(&mut scope, "console").unwrap();
			context
				.global(&mut scope)
				.set(&mut scope, console_string.into(), console.into());

			// get default export from page
			let page_module_namespace =
				run_module(&mut scope, self.fs.as_ref(), page_js_url.clone()).unwrap();
			let document_module_namespace =
				run_module(&mut scope, self.fs.as_ref(), document_js_url).unwrap();
			let default_string = v8::String::new(&mut scope, "default").unwrap().into();
			let page_module_default_export = page_module_namespace
				.get(&mut scope, default_string)
				.ok_or_else(|| format_err!("failed to find default export of page {}", page_js_url))
				.unwrap();

			// get default and renderPage export from document
			let document_module_default_export = document_module_namespace
				.get(&mut scope, default_string)
				.ok_or_else(|| format_err!("failed to find default export from document"))
				.unwrap();
			let render_page_string = v8::String::new(&mut scope, "renderPage").unwrap().into();
			let pinwheel_module_render_page_export = document_module_namespace
				.get(&mut scope, render_page_string)
				.ok_or_else(|| format_err!("failed to find renderPage export from document"))
				.unwrap();
			if !pinwheel_module_render_page_export.is_function() {
				return Err(format_err!(
					"renderPage export of document is not a function"
				));
			}
			let pinwheel_module_render_page_function: v8::Local<v8::Function> =
				unsafe { v8::Local::cast(pinwheel_module_render_page_export) };

			// send the props to v8
			let json = serde_json::to_string(&props).unwrap();
			let json = v8::String::new(&mut scope, &json).unwrap();
			let props = v8::json::parse(&mut scope, json).unwrap();
			let undefined = v8::undefined(&mut scope).into();

			let client_js_pathname = if let Some(client_js_pathname) = client_js_pathname {
				v8::String::new(&mut scope, client_js_pathname.to_str().unwrap())
					.unwrap()
					.into()
			} else {
				v8::undefined(&mut scope).into()
			};

			// call renderPage to render the page
			let mut try_catch_scope = v8::TryCatch::new(&mut scope);
			let html = pinwheel_module_render_page_function.call(
				&mut try_catch_scope,
				undefined,
				&[
					document_module_default_export,
					page_module_default_export,
					props,
					client_js_pathname,
				],
			);
			if try_catch_scope.has_caught() {
				let exception = try_catch_scope.exception().unwrap();
				let mut scope = v8::HandleScope::new(&mut try_catch_scope);
				print_error(&mut scope, exception);
				return Err(format_err!(""));
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

	pub async fn handle(&self, request: Request<Body>) -> Response<Body> {
		let method = request.method();
		let uri = request.uri();
		let path_and_query = uri.path_and_query().unwrap();
		let path = path_and_query.path();
		// serve static files from pinwheel
		let mut static_path = path.to_string();
		if static_path.ends_with('/') {
			static_path.push_str("index.html");
		}
		let static_path = static_path.strip_prefix('/').unwrap();
		// serve from the static dir in dev
		if let Some(src_dir) = self.src_dir.as_ref() {
			let static_path = src_dir.join("static").join(static_path);
			if static_path.exists() {
				let body = std::fs::read(&static_path).unwrap();
				let mut response = Response::builder();
				if let Some(content_type) = content_type(static_path.to_str().unwrap()) {
					response = response.header(header::CONTENT_TYPE, content_type);
				}
				let response = response.body(Body::from(body)).unwrap();
				return response;
			}
		}
		// serve from the out_dir
		let url = Url::parse(&format!("dst:/{}", static_path)).unwrap();
		if self.fs.exists(&url) {
			let data = self.fs.read(&url).unwrap();
			let mut response = Response::builder();
			if let Some(content_type) = content_type(static_path) {
				response = response.header("content-type", content_type);
			}
			let response = response.body(Body::from(data)).unwrap();
			return response;
		}
		let html = self.render(path, serde_json::Value::Null).unwrap();
		let response = Response::builder()
			.status(StatusCode::OK)
			.body(Body::from(html))
			.unwrap();
		eprintln!("{} {} {}", method, path, response.status().as_u16());
		response
	}
}

fn content_type(path: &str) -> Option<&'static str> {
	if path.ends_with(".js") {
		Some("text/javascript")
	} else if path.ends_with(".svg") {
		Some("image/svg+xml")
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
	let state = State {
		module_handles: Vec::new(),
	};
	isolate.set_slot(Rc::new(RefCell::new(state)));
	RefCell::new(isolate)
});

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
	fs: &dyn VirtualFileSystem,
	url: Url,
) -> Result<v8::Local<'s, v8::Object>> {
	let module_id = load_module(scope, fs, url).unwrap();
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
		print_error(&mut scope, exception);
	}
	let _ = module.evaluate(&mut try_catch_scope);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		print_error(&mut scope, exception);
	}
	drop(try_catch_scope);
	let namespace = module.get_module_namespace();
	let object = namespace.to_object(scope).unwrap();
	Ok(object)
}

/// load a module at the specified path and return the module id
fn load_module(scope: &mut v8::HandleScope, fs: &dyn VirtualFileSystem, url: Url) -> Result<i32> {
	// return the id for an existing module
	// if a module at the specified path
	// has alread been loaded
	let state = get_state(scope);
	let state = state.borrow();
	let existing_module = get_module_handle_for_url(&state, &url);
	if let Some(existing_module) = existing_module {
		return Ok(existing_module.id);
	}
	drop(state);

	// define the origin
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

	// read the source
	let code = fs.read(&url).unwrap();
	let code = std::str::from_utf8(code.as_ref()).unwrap();
	let source = v8::script_compiler::Source::new(v8::String::new(scope, code).unwrap(), &origin);

	// read the source map
	let source_map_url = match sourcemap::locate_sourcemap_reference_slice(code.as_bytes()).unwrap()
	{
		Some(s) => Some(url.join(s.get_url()).unwrap()),
		None => None,
	};
	let source_map = if let Some(source_map_url) = source_map_url {
		let source_map = fs.read(&source_map_url).unwrap();
		let source_map = sourcemap::SourceMap::from_slice(source_map.as_ref()).unwrap();
		Some(source_map)
	} else {
		None
	};

	// compile the module
	let mut try_catch_scope = v8::TryCatch::new(scope);
	let module = v8::script_compiler::compile_module(&mut try_catch_scope, source);
	if try_catch_scope.has_caught() {
		let exception = try_catch_scope.exception().unwrap();
		let mut scope = v8::HandleScope::new(&mut try_catch_scope);
		print_error(&mut scope, exception);
	}
	let module = module.unwrap();
	drop(try_catch_scope);

	// register the module
	let id = module.get_identity_hash();
	let state = get_state(scope);
	let mut state = state.borrow_mut();
	let global_module = v8::Global::<v8::Module>::new(scope, module);
	state.module_handles.push(ModuleHandle {
		id,
		url,
		module: global_module,
		source_map,
	});
	drop(state);

	// load each of the module's dependencies recursively
	for i in 0..module.get_module_requests_length() {
		let module_request = module.get_module_request(i);
		let specifier = module_request.to_rust_string_lossy(scope);
		let state = get_state(scope);
		let state = state.borrow();
		let referrer_url = &get_module_handle_with_id(&state, id).unwrap().url;
		let url = referrer_url.join(&specifier).unwrap();
		drop(state);
		load_module(scope, fs, url).unwrap();
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

fn print_error(scope: &mut v8::HandleScope, exception: v8::Local<v8::Value>) {
	let message = exception
		.to_string(scope)
		.unwrap()
		.to_rust_string_lossy(scope);
	eprintln!("{}", message);
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
			.lookup_token((source_line - 1) as u32, (source_column - 1) as u32)
			.unwrap();
		eprintln!(
			"{}:{}:{} -> {}:{}:{}",
			stack_trace_frame
				.get_script_name(scope)
				.unwrap()
				.to_rust_string_lossy(scope),
			stack_trace_frame.get_line_number(),
			stack_trace_frame.get_column(),
			token.get_source().unwrap_or("<unknown>"),
			token.get_src_line() + 1,
			token.get_src_col() + 1,
		);
	}
}

trait VirtualFileSystem: Send + Sync {
	fn exists(&self, url: &Url) -> bool;
	fn read(&self, url: &Url) -> Result<Cow<'static, [u8]>>;
}

struct RealFileSystem {
	dst_dir: PathBuf,
}

impl VirtualFileSystem for RealFileSystem {
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

impl VirtualFileSystem for IncludedFileSystem {
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
	// collect all pages in the pages directory
	let mut page_entries = Vec::new();
	let pages_dir = src_dir.join("pages");
	for path in walkdir::WalkDir::new(&pages_dir) {
		let path = path.unwrap();
		let path = path.path();
		match path.file_stem().unwrap().to_str().unwrap() {
			"static" | "server" => {
				let page_entry = path
					.strip_prefix(&pages_dir)
					.unwrap()
					.parent()
					.unwrap()
					.to_owned()
					.to_str()
					.unwrap()
					.to_owned()
					.into();
				page_entries.push(page_entry)
			}
			_ => {}
		}
	}
	// build the pages
	esbuild_pages(src_dir, dst_dir, &page_entries).unwrap();
	// copy static files
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
	// statically render pages
	let pinwheel = Pinwheel {
		src_dir: None,
		dst_dir: None,
		fs: Box::new(RealFileSystem {
			dst_dir: dst_dir.to_owned(),
		}),
	};
	for page_entry in page_entries {
		let mut pagename = String::from("/") + &page_entry;
		if pagename.ends_with("/index") {
			pagename = pagename.strip_suffix("index").unwrap().to_string();
		}
		let html = pinwheel.render(&pagename, serde_json::Value::Null)?;
		let html_path = dst_dir.join(page_entry.to_string() + ".html");
		let html_parent = html_path.parent().unwrap();
		std::fs::create_dir_all(html_parent).unwrap();
		std::fs::write(html_path, html).unwrap();
	}
	Ok(())
}

pub fn esbuild_single_page(root_dir: &Path, out_dir: &Path, page_entry: &str) -> Result<()> {
	esbuild_pages(root_dir, out_dir, &[page_entry.into()])
}

pub fn esbuild_pages(root_dir: &Path, out_dir: &Path, page_entries: &[Cow<str>]) -> Result<()> {
	// remove the out_dir if it exists and create it
	if out_dir.exists() {
		std::fs::remove_dir_all(&out_dir).unwrap();
	}
	std::fs::create_dir_all(&out_dir).unwrap();
	let manifest_path = out_dir.join("manifest.json");
	let document_source_path = root_dir.join("document.tsx");
	let mut args = vec![
		"run".to_string(),
		"-s".to_string(),
		"esbuild".to_string(),
		"--format=esm".to_string(),
		"--minify".to_string(),
		"--bundle".to_string(),
		"--splitting".to_string(),
		"--resolve-extensions=.ts,.tsx,.svg,.png".to_string(),
		"--loader:.svg=dataurl".to_string(),
		"--loader:.png=file".to_string(),
		"--sourcemap".to_string(),
		format!("--metafile={}", manifest_path.display()),
		format!("--outdir={}", out_dir.display()),
		format!("{}", document_source_path.display()),
	];
	for page_entry in page_entries {
		let page_source_path = root_dir
			.join("pages")
			.join(page_entry.as_ref())
			.join("static.tsx");
		let client_js_path = root_dir
			.join("pages")
			.join(page_entry.as_ref())
			.join("client.tsx");
		args.push(format!("{}", page_source_path.display()));
		if client_js_path.exists() {
			args.push(format!("{}", client_js_path.display()));
		}
	}
	let mut process = std::process::Command::new("yarn")
		.args(&args)
		.spawn()
		.unwrap();
	let status = process.wait().unwrap();
	if !status.success() {
		return Err(format_err!("esbuild {}", status.to_string()));
	}
	// concat css
	if std::env::current_dir()
		.unwrap()
		.components()
		.last()
		.unwrap()
		== std::path::Component::Normal(std::ffi::OsStr::new("www"))
	{
		let output = std::process::Command::new("fd")
			.args(&["-e", "css", ".", "../ui", ".", "-x", "cat"])
			.output()
			.unwrap();
		std::fs::write(out_dir.join("tangram.css"), output.stdout).unwrap();
	} else if std::env::current_dir()
		.unwrap()
		.components()
		.last()
		.unwrap()
		== std::path::Component::Normal(std::ffi::OsStr::new("tangram"))
	{
		let output = std::process::Command::new("fd")
			.args(&["-e", "css", ".", "ui", "app", "-x", "cat"])
			.output()
			.unwrap();
		std::fs::write(out_dir.join("tangram.css"), output.stdout).unwrap();
	} else {
		panic!()
	}
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
