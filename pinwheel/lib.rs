use self::esbuild::esbuild_single_page;
use anyhow::{format_err, Result};
use rusty_v8 as v8;
use sourcemap::SourceMap;
use std::{borrow::Cow, cell::RefCell, path::PathBuf, rc::Rc};
use url::Url;

mod esbuild;

pub use esbuild::esbuild_all_pages as build;

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

	pub async fn render<T>(&self, pagename: &str, props: T) -> Result<String>
	where
		T: serde::Serialize,
	{
		let page_entry = if pagename.ends_with('/') {
			pagename.to_string() + "index"
		} else {
			pagename.to_string()
		};
		let page_entry = page_entry.strip_prefix('/').unwrap();
		if self.src_dir.is_some() {
			esbuild_single_page(
				self.src_dir.as_ref().unwrap(),
				self.dst_dir.as_ref().unwrap(),
				page_entry,
			)?;
		}
		let pinwheel_js_url = Url::parse("dst:/pinwheel.js")?;
		let document_js_url = Url::parse("dst:/document.js")?;
		let page_js_url =
			Url::parse("dst:/")?.join(&("pages/".to_string() + page_entry + "/page.js"))?;
		let client_js_url = Url::parse("dst:/")?
			.join("pages")?
			.join(&("pages/".to_string() + page_entry + "/client.js"))?;
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

			let pinwheel_module_namespace =
				run_module(&mut scope, self.fs.as_ref(), pinwheel_js_url)?;
			let render_page_string = v8::String::new(&mut scope, "renderPage").unwrap().into();
			let pinwheel_module_render_page_export = pinwheel_module_namespace
				.get(&mut scope, render_page_string)
				.ok_or_else(|| format_err!("failed to find renderPage export of pinwheel.js"))?;
			if !pinwheel_module_render_page_export.is_function() {
				return Err(format_err!(
					"renderPage export of pinwheel.js is not a function"
				));
			}
			let pinwheel_module_render_page_function: v8::Local<v8::Function> =
				unsafe { v8::Local::cast(pinwheel_module_render_page_export) };

			let page_module_namespace =
				run_module(&mut scope, self.fs.as_ref(), page_js_url.clone())?;
			let document_namespace = run_module(&mut scope, self.fs.as_ref(), document_js_url)?;
			let default_string = v8::String::new(&mut scope, "default").unwrap().into();
			let page_module_default_export = page_module_namespace
				.get(&mut scope, default_string)
				.ok_or_else(|| {
					format_err!("failed to find default export of page {}", page_js_url)
				})?;
			let document_module_default_export = document_namespace
				.get(&mut scope, default_string)
				.ok_or_else(|| format_err!("failed to find default export of document.js"))?;

			// send the props to v8
			let json = serde_json::to_string(&props)?;
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

	pub fn serve(&self, path: &str) -> Option<Cow<'static, [u8]>> {
		let mut path = path.to_string();
		if path.ends_with('/') {
			path.push_str("index.html");
		}
		let path = path.strip_prefix('/').unwrap();
		if let Some(src_dir) = self.src_dir.as_ref() {
			let static_path = src_dir.join("static").join(path);
			if static_path.exists() {
				return Some(std::fs::read(static_path).unwrap().into());
			}
		}
		let url = Url::parse(&format!("dst:/{}", path)).unwrap();
		if self.fs.exists(&url) {
			return Some(self.fs.read(&url).unwrap());
		}
		None
	}
}

fn init_v8() {
	static V8_INIT: std::sync::Once = std::sync::Once::new();
	V8_INIT.call_once(|| {
		let platform = v8::new_default_platform().unwrap();
		v8::V8::initialize_platform(platform);
		v8::V8::initialize();
	});
}

thread_local!(static THREAD_LOCAL_ISOLATE: RefCell<v8::OwnedIsolate> = {
	init_v8();
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
	let code = fs.read(&url)?;
	let code = std::str::from_utf8(code.as_ref())?;
	let source = v8::script_compiler::Source::new(v8::String::new(scope, code).unwrap(), &origin);

	// read the source map
	let source_map_url = match sourcemap::locate_sourcemap_reference_slice(code.as_bytes())? {
		Some(s) => Some(url.join(s.get_url())?),
		None => None,
	};
	let source_map = if let Some(source_map_url) = source_map_url {
		let source_map = fs.read(&source_map_url)?;
		let source_map = sourcemap::SourceMap::from_slice(source_map.as_ref())?;
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
		let url = referrer_url.join(&specifier)?;
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
			.lookup_token(source_line as u32, source_column as u32)
			.unwrap();
		eprintln!(
			"{} {} {} {}",
			stack_trace_frame
				.get_script_name(scope)
				.unwrap()
				.to_rust_string_lossy(scope),
			stack_trace_frame.get_line_number(),
			stack_trace_frame.get_column(),
			token,
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
		self.dst_dir.join(url.path()).exists()
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
