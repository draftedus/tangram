use backtrace::Backtrace;
use futures::FutureExt;
use pinwheel::Pinwheel;
use std::{
	borrow::Cow, cell::RefCell, collections::BTreeMap, convert::Infallible,
	panic::AssertUnwindSafe, str::FromStr, sync::Arc,
};
use tangram_util::{err, error::Result};
use url::Url;

mod api;
pub mod common;
mod layouts;
mod migrations;
mod pages;
mod production_metrics;
mod production_stats;

pub struct Options {
	pub auth_enabled: bool,
	pub cookie_domain: Option<String>,
	pub database_url: Url,
	pub database_max_connections: Option<u32>,
	pub host: std::net::IpAddr,
	pub port: u16,
	pub sendgrid_api_token: Option<String>,
	pub stripe_publishable_key: Option<String>,
	pub stripe_secret_key: Option<String>,
	pub url: Option<Url>,
}

pub struct Context {
	pub options: Options,
	pub pinwheel: pinwheel::Pinwheel,
	pub pool: sqlx::AnyPool,
}

#[allow(clippy::cognitive_complexity)]
async fn handle(
	request: http::Request<hyper::Body>,
	context: Arc<Context>,
) -> http::Response<hyper::Body> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	let query = path_and_query.query();
	let path_components: Vec<_> = path.split('/').skip(1).collect();
	let search_params: Option<BTreeMap<String, String>> = query.map(|search_params| {
		url::form_urlencoded::parse(search_params.as_bytes())
			.into_owned()
			.collect()
	});
	let result = match (&method, path_components.as_slice()) {
		(&http::Method::GET, &["health"]) => self::api::health::get(&context, request).await,
		(&http::Method::POST, &["track"]) => self::api::track::post(&context, request).await,
		(&http::Method::GET, &["login"]) => self::pages::login::get(&context, request, search_params).await,
		(&http::Method::POST, &["login"]) => self::pages::login::post(&context, request).await,
		(&http::Method::GET, &[""]) => self::pages::index::get(&context, request).await,
		(&http::Method::POST, &[""]) => self::pages::index::post(&context, request).await,
		(&http::Method::GET, &["repos", "new"]) => self::pages::repos::new::get(&context, request).await,
		(&http::Method::POST, &["repos", "new"]) => self::pages::repos::new::post(&context, request).await,
		(&http::Method::GET, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::get(&context, request, repo_id).await
		}
		(&http::Method::POST, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::post(&context, request, repo_id).await
		}
		(&http::Method::GET, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::get(&context, request, repo_id).await
		}
		(&http::Method::POST, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::post(&context, request, repo_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			self::pages::repos::_repo_id::models::_model_id::index::get(&context, request, model_id).await
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id]) => {
			self::pages::repos::_repo_id::models::_model_id::post(&context, request, model_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			self::pages::repos::_repo_id::models::_model_id::download(&context, request, model_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::index::get(
				&context, request, model_id,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::columns::_column_name::get(
				&context,
				request,
				model_id,
				column_name,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_importances"]) => {
			self::pages::repos::_repo_id::models::_model_id::training_importances::get(
				&context, request, model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "prediction"]) => {
			self::pages::repos::_repo_id::models::_model_id::prediction::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::index::get(
				&context, request, model_id,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::class_metrics::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "precision_recall"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::precision_recall::get(
				&context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::roc::get(
				&context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			self::pages::repos::_repo_id::models::_model_id::tuning::get(&context, request, model_id)
				.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::index::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::index::post(
				&context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", "predictions", identifier]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::predictions::_identifier::get(
				&context,
				request,
				model_id,
				identifier,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_stats::index::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "production_stats", "columns", column_name],
		) => {
			self::pages::repos::_repo_id::models::_model_id::production_stats::columns::_column_name::get(
				&context,
				request,
				model_id,
				column_name,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_metrics::index::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "production_metrics", "class_metrics"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::production_metrics::class_metrics::get(
				&context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["user"]) => self::pages::user::get(&context, request).await,
		(&http::Method::POST, &["user"]) => self::pages::user::post(&context, request).await,
		(&http::Method::GET, &["organizations", "new"]) => {
			self::pages::organizations::new::get(&context, request).await
		}
		(&http::Method::POST, &["organizations", "new"]) => {
			self::pages::organizations::new::post(&context, request).await
		}
		(&http::Method::GET, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::get(&context, request, organization_id)
				.await
		}
		(&http::Method::POST, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::post(&context, request, organization_id)
				.await
		}
		(&http::Method::GET, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::get(&context, request, organization_id)
				.await
		}
		(&http::Method::GET, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::get(
				&context,
				request,
				organization_id,
			)
			.await
		}
		(&http::Method::POST, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::post(
				&context,
				request,
				organization_id,
			)
			.await
		}
		(&http::Method::POST, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::post(&context, request, organization_id)
				.await
		}
		_ => context.pinwheel.handle(request).await,
	};
	let response = match result {
		Ok(response) => response,
		Err(error) => {
			eprintln!("{}", error);
			let body: Cow<str> = if cfg!(debug_assertions) {
				format!("{}", error).into()
			} else {
				"internal server error".into()
			};
			http::Response::builder()
				.status(http::StatusCode::INTERNAL_SERVER_ERROR)
				.body(hyper::Body::from(body))
				.unwrap()
		}
	};
	eprintln!("{} {} {}", method, path, response.status());
	response
}

pub fn run(options: Options) -> Result<()> {
	tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_impl(options))
}

async fn run_impl(options: Options) -> Result<()> {
	// Create the pinwheel.
	#[cfg(debug_assertions)]
	fn pinwheel() -> Pinwheel {
		Pinwheel::dev(
			std::path::PathBuf::from("app"),
			std::path::PathBuf::from("build/pinwheel/app"),
		)
	}
	#[cfg(not(debug_assertions))]
	fn pinwheel() -> Pinwheel {
		Pinwheel::prod(include_dir::include_dir!("../build/pinwheel/app"))
	}
	let pinwheel = pinwheel();
	// Configure the database pool.
	let database_url = options.database_url.to_string();
	let (pool_options, pool_max_connections) = if database_url.starts_with("sqlite:") {
		let pool_options = sqlx::any::AnyConnectOptions::from(
			sqlx::sqlite::SqliteConnectOptions::from_str(&database_url)?
				.create_if_missing(true)
				.foreign_keys(true)
				.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
		);
		let pool_max_connections = options.database_max_connections.unwrap_or(1);
		(pool_options, pool_max_connections)
	} else if database_url.starts_with("postgres:") {
		let pool_options = sqlx::any::AnyConnectOptions::from(
			sqlx::postgres::PgConnectOptions::from_str(&database_url)?,
		);
		let pool_max_connections = options.database_max_connections.unwrap_or(10);
		(pool_options, pool_max_connections)
	} else {
		return Err(err!(
			"DATABASE_URL must be a sqlite or postgres database url"
		));
	};
	let pool = sqlx::any::AnyPoolOptions::new()
		.max_connections(pool_max_connections)
		.connect_with(pool_options)
		.await?;
	// Run any pending migrations.
	migrations::run(&pool).await?;
	// Run the server.
	tokio::task_local! {
		static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, Backtrace)>>;
	}
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
			panic_message_and_backtrace.borrow_mut().replace(value);
		})
	}));
	let context = Arc::new(Context {
		options,
		pinwheel,
		pool,
	});
	let service = hyper::service::make_service_fn(|_| {
		let context = context.clone();
		async move {
			Ok::<_, Infallible>(hyper::service::service_fn(move |request| {
				let method = request.method().clone();
				let path = request.uri().path_and_query().unwrap().path().to_owned();
				let context = context.clone();
				PANIC_MESSAGE_AND_BACKTRACE.scope(RefCell::new(None), async move {
					let response = AssertUnwindSafe(handle(request, context))
						.catch_unwind()
						.await
						.unwrap_or_else(|_| {
							let backtrace =
								PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
									let panic_message_and_backtrace =
										panic_message_and_backtrace.borrow();
									let (message, backtrace) =
										panic_message_and_backtrace.as_ref().unwrap();
									format!("{}\n{:?}", message, backtrace)
								});
							eprintln!("{} {} 500", method, path);
							http::Response::builder()
								.status(http::StatusCode::INTERNAL_SERVER_ERROR)
								.body(hyper::Body::from(backtrace))
								.unwrap()
						});
					Ok::<_, Infallible>(response)
				})
			}))
		}
	});
	let addr = std::net::SocketAddr::new(context.options.host, context.options.port);
	let listener = std::net::TcpListener::bind(&addr)?;
	eprintln!("🚀 serving on port {}", context.options.port);
	hyper::Server::from_tcp(listener)?.serve(service).await?;
	std::panic::set_hook(hook);
	Ok(())
}
