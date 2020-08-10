#![allow(non_snake_case)]

use self::error::Error;
use anyhow::Result;
use futures::FutureExt;
use hyper::{header, service::service_fn, Body, Method, Request, Response, StatusCode};
use pinwheel::Pinwheel;
use std::{collections::BTreeMap, panic::AssertUnwindSafe, path::PathBuf, str::FromStr, sync::Arc};

mod cookies;
mod error;
mod helpers;
mod migrations;
mod model;
mod monitor_event;
mod pages;
mod production_metrics;
mod production_stats;
mod time;
mod track;
mod types;

pub use helpers::user;

pub struct Context {
	pinwheel: Pinwheel,
	auth_enabled: bool,
	cookie_domain: Option<String>,
	sendgrid_api_token: Option<String>,
	stripe_secret_key: Option<String>,
	app_url: Option<String>,
	pool: sqlx::AnyPool,
}

#[allow(clippy::cognitive_complexity)]
async fn handle(
	request: Request<Body>,
	context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
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
		(&Method::GET, &["repos", _repo_id, "models", "new"]) => {
			pages::repos::_repo_id::models::new::get(request, &context).await
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
		(&Method::GET, &["repos", _repo_id, "models", model_id, "introspect"]) => {
			pages::repos::_repo_id::models::_model_id::introspect::get(request, &context, model_id)
				.await
		}
		(&Method::GET, &["repos", _repo_id, "models", model_id, "predict"]) => {
			pages::repos::_repo_id::models::_model_id::predict::get(request, &context, model_id)
				.await
		}
		(&Method::POST, &["repos", _repo_id, "models", model_id, "predict"]) => {
			pages::repos::_repo_id::models::_model_id::predict::post(request, &context, model_id)
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
		(&Method::GET, &["user", ""]) => pages::user::index::get(request, &context).await,
		(&Method::POST, &["user", ""]) => pages::user::index::post(request, &context).await,
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
		_ => Ok(context.pinwheel.handle(request).await),
	};
	let response = match result {
		Ok(r) => r,
		Err(error) => {
			if let Some(error) = error.downcast_ref::<Error>() {
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
	Ok(response)
}

pub async fn start() -> Result<()> {
	// get host and port
	let host = std::env::var("HOST")
		.map(|host| host.parse().expect("HOST environment variable invalid"))
		.unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
	let port = std::env::var("PORT")
		.map(|port| port.parse().expect("PORT environment variable invalid"))
		.unwrap_or_else(|_| 80);
	let addr = std::net::SocketAddr::new(host, port);

	// configure the database pool
	let database_url =
		std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
	let database_pool_max_size: u32 = std::env::var("DATABASE_POOL_MAX_SIZE")
		.map(|s| {
			s.parse()
				.expect("DATABASE_POOL_MAX_SIZE environment variable invalid")
		})
		.unwrap_or(10);
	// let database_cert = std::env::var("DATABASE_CERT").ok().map(|c| c.into_bytes());
	// let database_config = database_url.parse().unwrap();
	// let database_pool = if let Some(database_cert) = database_cert {
	// 	let tls = MakeTlsConnector::new(
	// 		TlsConnector::builder()
	// 			.add_root_certificate(Certificate::from_pem(&database_cert).unwrap())
	// 			.build()
	// 			.unwrap(),
	// 	);
	// 	let database_manager = Manager::new(database_config, tls);
	// 	Pool::new(database_manager, database_pool_max_size)
	// } else {
	// 	let database_manager = Manager::new(database_config, postgres::NoTls);
	// 	Pool::new(database_manager, database_pool_max_size)
	// };
	let options = match database_url {
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
		.connect_with(options)
		.await?;

	// run any pending migrations
	migrations::run(&pool).await?;

	// create the pinwheel
	#[cfg(debug_assertions)]
	fn pinwheel() -> Pinwheel {
		Pinwheel::dev(PathBuf::from("app"), PathBuf::from("target/js"))
	}
	#[cfg(not(debug_assertions))]
	fn pinwheel() -> Pinwheel {
		Pinwheel::prod(include_dir::include_dir!("../target/js"))
	}
	let pinwheel = pinwheel();

	// create the context
	let cookie_domain = std::env::var("COOKIE_DOMAIN").ok();
	let auth_enabled = std::env::var("AUTH_ENABLED")
		.map(|v| v == "1")
		.unwrap_or(false);
	let sendgrid_api_token = std::env::var("SENDGRID_API_TOKEN").ok();
	if auth_enabled && sendgrid_api_token.is_none() {
		panic!("SENDGRID_API_TOKEN environment variable must be set when AUTH_ENABLED = 1");
	}
	let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY").ok();
	let app_url = std::env::var("APP_URL").ok();
	let context = Arc::new(Context {
		pinwheel,
		auth_enabled,
		cookie_domain,
		sendgrid_api_token,
		stripe_secret_key,
		app_url,
		pool,
	});

	// start the server
	let listener = std::net::TcpListener::bind(&addr).unwrap();
	let mut listener = tokio::net::TcpListener::from_std(listener).unwrap();
	let http = hyper::server::conn::Http::new();
	eprintln!("🚀 serving on port {}", port);

	// wait for and handle each connection
	loop {
		let result = listener.accept().await;
		let (socket, _) = match result {
			Ok(s) => s,
			Err(e) => {
				eprintln!("tcp error: {}", e);
				continue;
			}
		};
		let context = context.clone();
		tokio::spawn(
			http.serve_connection(
				socket,
				service_fn(move |request| {
					let context = context.clone();
					AssertUnwindSafe(handle(request, context))
						.catch_unwind()
						.map(|result| match result {
							Err(_) => Ok(Response::builder()
								.status(StatusCode::INTERNAL_SERVER_ERROR)
								.body(Body::from("internal server error"))
								.unwrap()),
							Ok(response) => response,
						})
				}),
			)
			.map(|r| {
				if let Err(e) = r {
					eprintln!("http error: {}", e);
				}
			}),
		);
	}
}
