use crate::error::Result;
use std::{
	cell::RefCell, convert::Infallible, future::Future, panic::AssertUnwindSafe, path::Path,
	sync::Arc,
};
use tangram_deps::{
	backtrace::Backtrace,
	futures::FutureExt,
	hex, http, hyper,
	include_out_dir::Dir,
	sha2::{self, Digest},
	tokio,
};

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

pub async fn serve_from_dir(dir: &Path, path: &str) -> Result<Option<http::Response<hyper::Body>>> {
	let static_path = dir.join(path.strip_prefix('/').unwrap());
	let static_path_exists = std::fs::metadata(&static_path)
		.map(|metadata| metadata.is_file())
		.unwrap_or(false);
	if !static_path_exists {
		return Ok(None);
	}
	let body = tokio::fs::read(&static_path).await?;
	let mut response = http::Response::builder();
	if let Some(content_type) = content_type(&static_path) {
		response = response.header(http::header::CONTENT_TYPE, content_type);
	}
	let response = response.body(hyper::Body::from(body)).unwrap();
	Ok(Some(response))
}

pub async fn serve_from_include_dir(
	dir: &Dir,
	path: &str,
) -> Result<Option<http::Response<hyper::Body>>> {
	let static_path = Path::new(path.strip_prefix('/').unwrap());
	let body = if let Some(data) = dir.read(&static_path) {
		data
	} else {
		return Ok(None);
	};
	let mut response = http::Response::builder();
	if let Some(content_type) = content_type(&static_path) {
		response = response.header(http::header::CONTENT_TYPE, content_type);
	}
	let response = response.body(hyper::Body::from(body)).unwrap();
	Ok(Some(response))
}

fn content_type(path: &std::path::Path) -> Option<&'static str> {
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

pub fn hash(s: &str) -> String {
	let mut hash: sha2::Sha256 = Digest::new();
	hash.update(s);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}

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
