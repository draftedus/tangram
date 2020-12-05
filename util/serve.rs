use std::{cell::RefCell, convert::Infallible, future::Future, panic::AssertUnwindSafe, sync::Arc};
use tangram_deps::{
	backtrace::Backtrace,
	futures::FutureExt,
	hex, http, hyper,
	sha2::{self, Digest},
	tokio,
};

#[macro_export]
macro_rules! asset {
	($asset_relative_path:literal) => {{
		let file_path = ::std::path::Path::new(file!());
		let asset_path = file_path.parent().unwrap().join($asset_relative_path);
		if cfg!(debug_assertions) {
			format!("/assets/{}", asset_path.display())
		} else {
			let extension = asset_path.extension().map(|e| e.to_str().unwrap()).unwrap();
			let hash = tangram_util::serve::hash(&asset_path.to_str().unwrap());
			format!("/assets/{}.{}", hash, extension)
			}
		}};
}

#[macro_export]
macro_rules! client {
	() => {{
		let file_path = ::std::path::Path::new(file!());
		let client_crate_manifest_path = file_path.parent().unwrap().join("client/Cargo.toml");
		let hash = tangram_util::serve::hash(client_crate_manifest_path.to_str().unwrap());
		format!("/js/{}.js", hash)
		}};
}

pub async fn serve<C, H, F>(
	host: std::net::IpAddr,
	port: u16,
	request_handler_context: C,
	request_handler: H,
) -> hyper::Result<()>
where
	C: Send + Sync + 'static,
	H: Fn(Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
	F: Future<Output = http::Response<hyper::Body>> + Send,
{
	// Create a task local that will store the panic message and backtrace if a panic occurs.
	tokio::task_local! {
		static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, Backtrace)>>;
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

// 	pub async fn handle(
// 		request: http::Request<hyper::Body>,
// 	) -> http::Response<hyper::Body> {
// 		let uri = request.uri();
// 		let path_and_query = uri.path_and_query().unwrap();
// 		let path = path_and_query.path();
// 		// Render static pages in dev.
// 		if let Pinwheel::Dev { src_dir, .. } = self.as_ref() {
// 			let page_entry = page_entry_for_pagename(path);
// 			let page_exists = js_sources_for_page_entry(src_dir, &page_entry).is_some();
// 			if page_exists {
// 				let html = self.render(path).unwrap();
// 				let response = http::Response::builder()
// 					.status(http::StatusCode::OK)
// 					.body(hyper::Body::from(html))
// 					.unwrap();
// 				return response;
// 			}
// 		}
// 		// Serve static files from the static directory in dev.
// 		if let Pinwheel::Dev { src_dir, .. } = self.as_ref() {
// 			let static_path = src_dir.join("static").join(path.strip_prefix('/').unwrap());
// 			let exists = std::fs::metadata(&static_path)
// 				.map(|metadata| metadata.is_file())
// 				.unwrap_or(false);
// 			if exists {
// 				let body = std::fs::read(&static_path).unwrap();
// 				let mut response = http::Response::builder();
// 				if let Some(content_type) = content_type(&static_path) {
// 					response = response.header(http::header::CONTENT_TYPE, content_type);
// 				}
// 				let response = response.body(hyper::Body::from(body)).unwrap();
// 				return response;
// 			}
// 		}
// 		// Serve assets from the src_dir in dev.
// 		if let Pinwheel::Dev { .. } = self.as_ref() {
// 			if let Some(path) = path.strip_prefix("/assets") {
// 				let asset_path = Path::new(path.strip_prefix('/').unwrap());
// 				let exists = std::fs::metadata(&asset_path)
// 					.map(|metadata| metadata.is_file())
// 					.unwrap_or(false);
// 				if exists {
// 					let body = std::fs::read(&asset_path).unwrap();
// 					let mut response = http::Response::builder();
// 					if let Some(content_type) = content_type(&asset_path) {
// 						response = response.header(http::header::CONTENT_TYPE, content_type);
// 					}
// 					let response = response.body(hyper::Body::from(body)).unwrap();
// 					return response;
// 				}
// 			}
// 		}
// 		// Serve from the dst_dir.
// 		let url = Url::parse(&format!("dst:{}", path)).unwrap();
// 		if self.fs().file_exists(&url) {
// 			let data = self.fs().read(&url).unwrap();
// 			let mut response = http::Response::builder();
// 			if let Some(content_type) = content_type(Path::new(path)) {
// 				response = response.header(http::header::CONTENT_TYPE, content_type);
// 			}
// 			let response = response.body(hyper::Body::from(data)).unwrap();
// 			return response;
// 		}
// 		// Otherwise, 404.
// 		http::Response::builder()
// 			.status(http::StatusCode::NOT_FOUND)
// 			.body(hyper::Body::from("not found"))
// 			.unwrap()
// 	}
// }

// pub fn build_client_crate(
// 	src_dir: &Path,
// 	client_crate_manifest_paths: &[PathBuf],
// 	cargo_wasm_dir: &Path,
// 	dev: bool,
// 	dst_dir: &Path,
// ) -> Result<()> {
// 	let output_wasm_dir = dst_dir.join("js");
// 	let client_crate_package_names = client_crate_manifest_paths
// 		.iter()
// 		.map(|client_crate_manifest_path| {
// 			let client_crate_manifest =
// 				std::fs::read_to_string(&src_dir.join(client_crate_manifest_path))?;
// 			let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
// 			let client_crate_name = client_crate_manifest
// 				.as_table()
// 				.unwrap()
// 				.get("package")
// 				.unwrap()
// 				.as_table()
// 				.unwrap()
// 				.get("name")
// 				.unwrap()
// 				.as_str()
// 				.unwrap()
// 				.to_owned();
// 			Ok(client_crate_name)
// 		})
// 		.collect::<Result<Vec<_>>>()?;
// 	let cmd = which("cargo")?;
// 	let mut args = vec![
// 		"build".to_owned(),
// 		"--target".to_owned(),
// 		"wasm32-unknown-unknown".to_owned(),
// 		"--target-dir".to_owned(),
// 		cargo_wasm_dir.to_str().unwrap().to_owned(),
// 	];
// 	if !dev {
// 		args.push("--release".to_owned())
// 	}
// 	for client_crate_package_name in client_crate_package_names.iter() {
// 		args.push("--package".to_owned());
// 		args.push(client_crate_package_name.clone());
// 	}
// 	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
// 	let status = process.wait()?;
// 	if !status.success() {
// 		return Err(err!("cargo {}", status.to_string()));
// 	}
// 	pzip!(client_crate_manifest_paths, client_crate_package_names).for_each(
// 		|(client_crate_manifest_path, client_crate_package_name)| {
// 			let hash = hash(client_crate_manifest_path.to_str().unwrap());
// 			let input_wasm_path = format!(
// 				"{}/wasm32-unknown-unknown/{}/{}.wasm",
// 				cargo_wasm_dir.to_str().unwrap(),
// 				if dev { "debug" } else { "release" },
// 				client_crate_package_name,
// 			);
// 			let output_wasm_path = output_wasm_dir.join(format!("{}.wasm", hash));
// 			// Do not re-run wasm-bindgen if the output wasm exists and is not older than the input wasm.
// 			let input_wasm_metadata = std::fs::metadata(&input_wasm_path).unwrap();
// 			let input_wasm_modified_time = input_wasm_metadata.modified().unwrap();
// 			if let Ok(output_wasm_metadata) = std::fs::metadata(&output_wasm_path) {
// 				let output_wasm_modified_time = output_wasm_metadata.modified().unwrap();
// 				if input_wasm_modified_time <= output_wasm_modified_time {
// 					return;
// 				}
// 			}
// 			wasm_bindgen_cli_support::Bindgen::new()
// 				.web(true)
// 				.unwrap()
// 				.keep_debug(dev)
// 				.remove_producers_section(true)
// 				.remove_name_section(true)
// 				.input_path(input_wasm_path)
// 				.out_name(&hash)
// 				.generate(&output_wasm_dir)
// 				.map_err(|error| err!(error))
// 				.unwrap();
// 		},
// 	);
// 	Ok(())
// }

pub fn hash(s: &str) -> String {
	let mut hash: sha2::Sha256 = Digest::new();
	hash.update(s);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}
