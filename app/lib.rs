use pinwheel::Pinwheel;
use std::{borrow::Cow, collections::BTreeMap, sync::Arc};
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

pub fn run(options: Options) -> Result<()> {
	tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_inner(options))
}

async fn run_inner(options: Options) -> Result<()> {
	// Create the pinwheel.
	let pinwheel = if cfg!(debug_assertions) {
		Pinwheel::dev(
			std::path::PathBuf::from("app"),
			std::path::PathBuf::from("build/pinwheel/app"),
		)
	} else {
		Pinwheel::prod(include_dir::include_dir!("../build/pinwheel/app"))
	};
	// Configure the database pool.
	let database_url = options.database_url.to_string();
	let (pool_options, pool_max_connections) = if database_url.starts_with("sqlite:") {
		let pool_options = database_url
			.parse::<sqlx::sqlite::SqliteConnectOptions>()?
			.create_if_missing(true)
			.foreign_keys(true)
			.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
			.into();
		let pool_max_connections = options.database_max_connections.unwrap_or(1);
		(pool_options, pool_max_connections)
	} else if database_url.starts_with("postgres:") {
		let pool_options = database_url
			.parse::<sqlx::postgres::PgConnectOptions>()?
			.into();
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
	// Serve!
	let host = options.host;
	let port = options.port;
	let context = Context {
		options,
		pinwheel,
		pool,
	};
	pinwheel::serve(host, port, context, handle).await?;
	Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn handle(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
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
	let context = &context;
	let result = match (&method, path_components.as_slice()) {
		(&http::Method::GET, &["health"]) => self::api::health::get(context, request).await,
		(&http::Method::POST, &["track"]) => self::api::track::post(context, request).await,
		(&http::Method::GET, &["login"]) => self::pages::login::get(context, request, search_params).await,
		(&http::Method::POST, &["login"]) => self::pages::login::post(context, request).await,
		(&http::Method::GET, &[""]) => self::pages::index::get(context, request).await,
		(&http::Method::POST, &[""]) => self::pages::index::post(context, request).await,
		(&http::Method::GET, &["repos", "new"]) => self::pages::repos::new::get(context, request).await,
		(&http::Method::POST, &["repos", "new"]) => self::pages::repos::new::post(context, request).await,
		(&http::Method::GET, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::get(context, request, repo_id).await
		}
		(&http::Method::POST, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::post(context, request, repo_id).await
		}
		(&http::Method::GET, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::get(context, request, repo_id).await
		}
		(&http::Method::POST, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::post(context, request, repo_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			self::pages::repos::_repo_id::models::_model_id::index::get(context, request, model_id).await
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id]) => {
			self::pages::repos::_repo_id::models::_model_id::post(context, request, model_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			self::pages::repos::_repo_id::models::_model_id::download(context, request, model_id).await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::index::get(
				context, request, model_id,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::columns::_column_name::get(
				context,
				request,
				model_id,
				column_name,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_importances"]) => {
			self::pages::repos::_repo_id::models::_model_id::training_importances::get(
				context, request, model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "prediction"]) => {
			self::pages::repos::_repo_id::models::_model_id::prediction::get(
				context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::index::get(
				context, request, model_id,
			)
			.await
		}
		(
			&http::Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::class_metrics::get(
				context,
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
				context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::roc::get(
				context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			self::pages::repos::_repo_id::models::_model_id::tuning::get(context, request, model_id)
				.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::index::get(
				context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::index::post(
				context,
				request,
				model_id,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", "predictions", identifier]) => {
			self::pages::repos::_repo_id::models::_model_id::production_predictions::predictions::_identifier::get(
				context,
				request,
				model_id,
				identifier,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_stats::index::get(
				context,
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
				context,
				request,
				model_id,
				column_name,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_metrics::index::get(
				context,
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
				context,
				request,
				model_id,
				search_params,
			)
			.await
		}
		(&http::Method::GET, &["user"]) => self::pages::user::get(context, request).await,
		(&http::Method::POST, &["user"]) => self::pages::user::post(context, request).await,
		(&http::Method::GET, &["organizations", "new"]) => {
			self::pages::organizations::new::get(context, request).await
		}
		(&http::Method::POST, &["organizations", "new"]) => {
			self::pages::organizations::new::post(context, request).await
		}
		(&http::Method::GET, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::get(context, request, organization_id)
				.await
		}
		(&http::Method::POST, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::post(context, request, organization_id)
				.await
		}
		(&http::Method::GET, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::get(context, request, organization_id)
				.await
		}
		(&http::Method::GET, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::get(
				context,
				request,
				organization_id,
			)
			.await
		}
		(&http::Method::POST, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::post(
				context,
				request,
				organization_id,
			)
			.await
		}
		(&http::Method::POST, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::post(context, request, organization_id)
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
