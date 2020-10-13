use self::{common::error::Error, context::Context};
use anyhow::{format_err, Result};
use futures::FutureExt;
use hyper::{
	header,
	service::{make_service_fn, service_fn},
	Body, Method, Request, Response, StatusCode,
};
use pinwheel::Pinwheel;
use std::{
	borrow::Cow, collections::BTreeMap, convert::Infallible, panic::AssertUnwindSafe,
	path::PathBuf, str::FromStr, sync::Arc,
};
use tangram_util::id::Id;
use url::Url;

mod api;
mod common;
mod migrations;
mod pages;
mod production_metrics;
mod production_stats;

pub struct Options {
	pub auth_enabled: bool,
	pub cookie_domain: Option<String>,
	pub database_url: Url,
	pub host: std::net::IpAddr,
	pub model: Option<PathBuf>,
	pub port: u16,
	pub sendgrid_api_token: Option<String>,
	pub stripe_publishable_key: Option<String>,
	pub stripe_secret_key: Option<String>,
	pub url: Option<Url>,
}

mod context {
	pub struct Context {
		pub options: super::Options,
		pub pinwheel: pinwheel::Pinwheel,
		pub pool: sqlx::AnyPool,
	}
}

#[allow(clippy::cognitive_complexity)]
async fn handle(request: Request<Body>, context: Arc<Context>) -> Response<Body> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	let path_components: Vec<_> = path.split('/').skip(1).collect();
	let search_params: Option<BTreeMap<String, String>> =
		path_and_query.query().map(|search_params| {
			url::form_urlencoded::parse(search_params.as_bytes())
				.into_owned()
				.collect()
		});
	let result = match (&method, path_components.as_slice()) {
		(&Method::GET, &["health"]) => self::api::health::get(request, &context).await,
		(&Method::POST, &["track"]) => self::api::track::post(request, context).await,
		(&Method::GET, &["login"]) => self::pages::login::get(request, context, search_params).await,
		(&Method::POST, &["login"]) => self::pages::login::post(request, &context).await,
		(&Method::GET, &[""]) => self::pages::index::get(request, &context).await,
		(&Method::POST, &[""]) => self::pages::index::post(request, &context).await,
		(&Method::GET, &["repos", "new"]) => self::pages::repos::new::get(request, &context).await,
		(&Method::POST, &["repos", "new"]) => self::pages::repos::new::post(request, &context).await,
		(&Method::GET, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::get(request, &context, repo_id).await
		}
		(&Method::POST, &["repos", repo_id, ""]) => {
			self::pages::repos::_repo_id::index::post(request, &context, repo_id).await
		}
		(&Method::GET, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::get(request, &context, repo_id).await
		}
		(&Method::POST, &["repos", repo_id, "models", "new"]) => {
			self::pages::repos::_repo_id::models::new::post(request, &context, repo_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			self::pages::repos::_repo_id::models::_model_id::index::get(request, &context, model_id).await
		}
		(&Method::POST, &["repos", _repo_id, "models", model_id]) => {
			self::pages::repos::_repo_id::models::_model_id::post(request, &context, model_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			self::pages::repos::_repo_id::models::_model_id::download(request, &context, model_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::index::get(
				request, &context, model_id,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_stats::columns::_column_name::get(
				request,
				&context,
				model_id,
				column_name,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "introspection"]) => {
			self::pages::repos::_repo_id::models::_model_id::introspection::get(
				request, &context, model_id,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "prediction"]) => {
			self::pages::repos::_repo_id::models::_model_id::prediction::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::index::get(
				request, &context, model_id,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::class_metrics::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_metrics::index::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "production_metrics", "class_metrics"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::production_metrics::class_metrics::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "precision_recall"],
		) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::precision_recall::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			self::pages::repos::_repo_id::models::_model_id::training_metrics::roc::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			self::pages::repos::_repo_id::models::_model_id::tuning::get(request, &context, model_id)
				.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			self::pages::repos::_repo_id::models::_model_id::production_stats::index::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "production_stats", "columns", column_name],
		) => {
			self::pages::repos::_repo_id::models::_model_id::production_stats::columns::_column_name::get(
				request,
				&context,
				model_id,
				column_name,
				search_params,
			)
			.await
		}
		(&Method::GET, &["user"]) => self::pages::user::get(request, &context).await,
		(&Method::POST, &["user"]) => self::pages::user::post(request, &context).await,
		(&Method::GET, &["organizations", "new"]) => {
			self::pages::organizations::new::get(request, &context).await
		}
		(&Method::POST, &["organizations", "new"]) => {
			self::pages::organizations::new::post(request, &context).await
		}
		(&Method::GET, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::get(request, &context, organization_id)
				.await
		}
		(&Method::POST, &["organizations", organization_id, ""]) => {
			self::pages::organizations::_organization_id::index::post(request, &context, organization_id)
				.await
		}
		(&Method::GET, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::get(request, &context, organization_id)
				.await
		}
		(&Method::GET, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::get(
				request,
				&context,
				organization_id,
			)
			.await
		}
		(&Method::POST, &["organizations", organization_id, "members", "new"]) => {
			self::pages::organizations::_organization_id::members::new::post(
				request,
				&context,
				organization_id,
			)
			.await
		}
		(&Method::POST, &["organizations", organization_id, "edit"]) => {
			self::pages::organizations::_organization_id::edit::post(request, &context, organization_id)
				.await
		}
		_ => context.pinwheel.handle(request).await,
	};
	let response = match result {
		Ok(r) => r,
		Err(error) => {
			if error.downcast_ref::<pinwheel::NotFoundError>().is_some() {
				Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body(Body::from("not found"))
					.unwrap()
			} else if let Some(error) = error.downcast_ref::<Error>() {
				match error {
					Error::BadRequest => Response::builder()
						.status(StatusCode::BAD_REQUEST)
						.body(Body::from("bad request"))
						.unwrap(),
					Error::Unauthorized => Response::builder()
						.status(StatusCode::SEE_OTHER)
						.header(header::LOCATION, "/login")
						.body(Body::from("unauthorized"))
						.unwrap(),
					Error::NotFound => Response::builder()
						.status(StatusCode::NOT_FOUND)
						.body(Body::from("not found"))
						.unwrap(),
					Error::ServiceUnavailable => Response::builder()
						.status(StatusCode::SERVICE_UNAVAILABLE)
						.body(Body::from("service unavailable"))
						.unwrap(),
				}
			} else {
				eprintln!("{}", error);
				let body: Cow<str> = if cfg!(debug_assertions) {
					error.to_string().into()
				} else {
					"internal server error".into()
				};
				Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(Body::from(body))
					.unwrap()
			}
		}
	};
	eprintln!("{} {} {}", method, path, response.status().as_u16());
	response
}

pub async fn run(options: Options) -> Result<()> {
	// Create the pinwheel.
	#[cfg(debug_assertions)]
	fn pinwheel() -> Pinwheel {
		Pinwheel::dev(
			std::path::PathBuf::from("app"),
			std::path::PathBuf::from("target/app"),
		)
	}
	#[cfg(not(debug_assertions))]
	fn pinwheel() -> Pinwheel {
		Pinwheel::prod(include_dir::include_dir!("../target/app"))
	}
	let pinwheel = pinwheel();
	// Configure the database pool.
	let database_url = options.database_url.to_string();
	let database_pool_max_size: u32 = std::env::var("DATABASE_POOL_MAX_SIZE")
		.map(|s| {
			s.parse()
				.expect("DATABASE_POOL_MAX_SIZE environment variable invalid")
		})
		.unwrap_or(10);
	let pool_options = match database_url {
		_ if database_url.starts_with("sqlite:") => sqlx::any::AnyConnectOptions::from(
			sqlx::sqlite::SqliteConnectOptions::from_str(&database_url)?
				.create_if_missing(true)
				.foreign_keys(true)
				.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
		),
		_ if database_url.starts_with("postgres:") => sqlx::any::AnyConnectOptions::from(
			sqlx::postgres::PgConnectOptions::from_str(&database_url)?,
		),
		_ => panic!("DATABASE_URL must be a sqlite or postgres database url"),
	};
	let pool = sqlx::any::AnyPoolOptions::new()
		.max_connections(database_pool_max_size)
		.connect_with(pool_options)
		.await?;
	// Run any pending migrations.
	migrations::run(&pool).await?;
	// If a model was included in the options, add it to the database now.
	if let Some(model_path) = &options.model {
		let mut db = pool.begin().await?;
		let repo_id = Id::new();
		let model_data = std::fs::read(model_path)?;
		let model = tangram_core::model::Model::from_slice(&model_data)?;
		let title = model_path
			.file_stem()
			.ok_or_else(|| format_err!("bad model path"))?
			.to_str()
			.ok_or_else(|| format_err!("bad model path"))?;
		crate::common::repos::create_root_repo(&mut db, repo_id, title).await?;
		crate::common::repos::add_model_version(&mut db, repo_id, model.id(), &model_data).await?;
		db.commit().await?;
	}
	// Run the server.
	let context = Arc::new(Context {
		options,
		pinwheel,
		pool,
	});
	let service = make_service_fn(|_| {
		let context = context.clone();
		async move {
			Ok::<_, Infallible>(service_fn(move |request| {
				let method = request.method().to_owned();
				let path = request.uri().path_and_query().unwrap().path().to_owned();
				let context = context.clone();
				async move {
					Ok::<_, Infallible>(
						AssertUnwindSafe(handle(request, context))
							.catch_unwind()
							.await
							.unwrap_or_else(|_| {
								eprintln!("{} {} 500", method, path);
								Response::builder()
									.status(StatusCode::INTERNAL_SERVER_ERROR)
									.body(Body::from("internal server error"))
									.unwrap()
							}),
					)
				}
			}))
		}
	});
	let addr = std::net::SocketAddr::new(context.options.host, context.options.port);
	let listener = std::net::TcpListener::bind(&addr)?;
	eprintln!("🚀 serving on port {}", context.options.port);
	hyper::Server::from_tcp(listener)?.serve(service).await?;
	Ok(())
}
