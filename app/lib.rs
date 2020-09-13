use self::error::Error;
use anyhow::Result;
use futures::FutureExt;
use hyper::{
	header,
	service::{make_service_fn, service_fn},
	Body, Method, Request, Response, StatusCode,
};
use pinwheel::Pinwheel;
use std::{
	collections::BTreeMap, convert::Infallible, panic::AssertUnwindSafe, str::FromStr, sync::Arc,
};
use url::Url;

mod common;
mod error;
mod migrations;
mod monitor_event;
mod pages;
mod production_metrics;
mod production_stats;
mod track;

pub struct AppOptions {
	pub auth_enabled: bool,
	pub cookie_domain: Option<String>,
	pub database_url: Option<Url>,
	pub host: std::net::IpAddr,
	pub port: u16,
	pub sendgrid_api_token: Option<String>,
	pub stripe_publishable_key: Option<String>,
	pub stripe_secret_key: Option<String>,
	pub url: Option<Url>,
}

pub struct Context {
	options: AppOptions,
	pinwheel: Pinwheel,
	pool: sqlx::AnyPool,
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
		(&Method::GET, &["health"]) => pages::health::get(request, &context).await,
		(&Method::POST, &["track"]) => track::track(request, context).await,
		(&Method::GET, &["login"]) => pages::login::get(request, context, search_params).await,
		(&Method::POST, &["login"]) => pages::login::post(request, &context).await,
		(&Method::GET, &[""]) => pages::index::get(request, &context).await,
		(&Method::POST, &[""]) => pages::index::post(request, &context).await,
		(&Method::GET, &["repos", "new"]) => pages::repos::new::get(request, &context).await,
		(&Method::POST, &["repos", "new"]) => pages::repos::new::post(request, &context).await,
		(&Method::GET, &["repos", repo_id, ""]) => {
			pages::repos::_repo_id::index::get(request, &context, repo_id).await
		}
		(&Method::POST, &["repos", repo_id, ""]) => {
			pages::repos::_repo_id::index::post(request, &context, repo_id).await
		}
		(&Method::GET, &["repos", repo_id, "models", "new"]) => {
			pages::repos::_repo_id::models::new::get(request, &context, repo_id).await
		}
		(&Method::POST, &["repos", repo_id, "models", "new"]) => {
			pages::repos::_repo_id::models::new::post(request, &context, repo_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			pages::repos::_repo_id::models::_model_id::index::get(request, &context, model_id).await
		}
		(&Method::POST, &["repos", _repo_id, "models", model_id]) => {
			pages::repos::_repo_id::models::_model_id::post(request, &context, model_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			pages::repos::_repo_id::models::_model_id::download(request, &context, model_id).await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			pages::repos::_repo_id::models::_model_id::training_stats::index::get(
				request, &context, model_id,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name],
		) => {
			pages::repos::_repo_id::models::_model_id::training_stats::columns::_column_name::get(
				request,
				&context,
				model_id,
				column_name,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "introspection"]) => {
			pages::repos::_repo_id::models::_model_id::introspection::get(
				request, &context, model_id,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "prediction"]) => {
			pages::repos::_repo_id::models::_model_id::prediction::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			pages::repos::_repo_id::models::_model_id::training_metrics::index::get(
				request, &context, model_id,
			)
			.await
		}
		(
			&Method::GET,
			&["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"],
		) => {
			pages::repos::_repo_id::models::_model_id::training_metrics::class_metrics::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			pages::repos::_repo_id::models::_model_id::production_metrics::index::get(
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
			pages::repos::_repo_id::models::_model_id::production_metrics::class_metrics::get(
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
			pages::repos::_repo_id::models::_model_id::training_metrics::precision_recall::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			pages::repos::_repo_id::models::_model_id::training_metrics::roc::get(
				request,
				&context,
				model_id,
				search_params,
			)
			.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			pages::repos::_repo_id::models::_model_id::tuning::get(request, &context, model_id)
				.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			pages::repos::_repo_id::models::_model_id::production_stats::index::get(
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
			pages::repos::_repo_id::models::_model_id::production_stats::columns::_column_name::get(
				request,
				&context,
				model_id,
				column_name,
				search_params,
			)
			.await
		}
		(&Method::GET, &["user"]) => pages::user::get(request, &context).await,
		(&Method::POST, &["user"]) => pages::user::post(request, &context).await,
		(&Method::GET, &["organizations", "new"]) => {
			pages::organizations::new::get(request, &context).await
		}
		(&Method::POST, &["organizations", "new"]) => {
			pages::organizations::new::post(request, &context).await
		}
		(&Method::GET, &["organizations", organization_id, ""]) => {
			pages::organizations::_organization_id::index::get(request, &context, organization_id)
				.await
		}
		(&Method::POST, &["organizations", organization_id, ""]) => {
			pages::organizations::_organization_id::index::post(request, &context, organization_id)
				.await
		}
		(&Method::GET, &["organizations", organization_id, "edit"]) => {
			pages::organizations::_organization_id::edit::get(request, &context, organization_id)
				.await
		}
		(&Method::GET, &["organizations", organization_id, "members", "new"]) => {
			pages::organizations::_organization_id::members::new::get(
				request,
				&context,
				organization_id,
			)
			.await
		}
		(&Method::POST, &["organizations", organization_id, "members", "new"]) => {
			pages::organizations::_organization_id::members::new::post(
				request,
				&context,
				organization_id,
			)
			.await
		}
		(&Method::POST, &["organizations", organization_id, "edit"]) => {
			pages::organizations::_organization_id::edit::post(request, &context, organization_id)
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
				Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(Body::from("internal server error"))
					.unwrap()
			}
		}
	};
	eprintln!("{} {} {}", method, path, response.status().as_u16());
	response
}

pub async fn run(options: AppOptions) -> Result<()> {
	// create the pinwheel
	#[cfg(debug_assertions)]
	fn pinwheel() -> Pinwheel {
		Pinwheel::dev(
			std::path::PathBuf::from("app"),
			std::path::PathBuf::from("target/app"),
		)
	}
	#[cfg(not(debug_assertions))]
	fn pinwheel() -> Pinwheel {
		Pinwheel::prod(include_dir::include_dir!("target/app"))
	}
	let pinwheel = pinwheel();

	// configure the database pool
	let database_url = options
		.database_url
		.as_ref()
		.map(|url| url.to_string())
		.unwrap_or_else(|| {
			let tangram_data_dir = dirs::data_dir()
				.expect("failed to find user data directory")
				.join("tangram");
			std::fs::create_dir_all(&tangram_data_dir).unwrap_or_else(|_| {
				panic!(
					"failed to create tangram data directory in {}",
					tangram_data_dir.display()
				)
			});
			let tangram_database_path = tangram_data_dir.join("tangram.db");
			format!(
				"sqlite:{}",
				tangram_database_path.to_str().unwrap().to_owned()
			)
		});
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

	// run any pending migrations
	migrations::run(&pool).await?;

	// run the server
	let context = Arc::new(Context {
		options,
		pinwheel,
		pool,
	});
	let service = make_service_fn(|_| {
		let context = context.clone();
		async move {
			Ok::<_, Infallible>(service_fn(move |request| {
				let context = context.clone();
				async move {
					Ok::<_, Infallible>(
						AssertUnwindSafe(handle(request, context))
							.catch_unwind()
							.await
							.unwrap_or_else(|_| {
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
