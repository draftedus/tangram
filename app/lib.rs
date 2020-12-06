use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, sync::Arc};
use tangram_app_common::Context;
use tangram_deps::{futures::FutureExt, http, hyper, sqlx, tokio, url};
use tangram_util::{err, error::Result, serve::serve};

mod migrations;

pub use tangram_app_common::Options;

pub fn run(options: Options) -> Result<()> {
	tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_inner(options))
}

async fn run_inner(options: Options) -> Result<()> {
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
	self::migrations::run(&pool).await?;
	// Start the server.
	let host = options.host;
	let port = options.port;
	let context = Context { options, pool };
	serve(host, port, context, request_handler).await?;
	Ok(())
}

async fn request_handler(
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
		(&http::Method::GET, &["health"]) => {
			tangram_api::health::get(
				context,
				request,
			).boxed()
		}
		(&http::Method::POST, &["track"]) => {
			tangram_api::track::post(
				context,
				request,
			).boxed()
		}
		(&http::Method::GET, &["login"]) => {
			tangram_app_pages_login::get(
				context,
				request,
				search_params
			).boxed()
		},
		(&http::Method::POST, &["login"]) => {
			tangram_app_pages_login::post(
				context,
				request,
			).boxed()
		}
		(&http::Method::GET, &[""]) => {
			tangram_app_pages_index::get(
				context,
				request,
			).boxed()
		},
		(&http::Method::POST, &[""]) => {
			tangram_app_pages_index::post(
				context,
				request,
			).boxed()
		}
		(&http::Method::GET, &["repos", "new"]) => {
			tangram_app_pages_repos_new::get(
				context,
				request,
			).boxed()
		}
		(&http::Method::POST, &["repos", "new"]) => {
			tangram_app_pages_repos_new::post(
				context,
				request,
			).boxed()
		}
		(&http::Method::GET, &["repos", repo_id, ""]) => {
			tangram_app_pages_repos_repo_id_index::get(
				context,
				request,
				repo_id,
			).boxed()
		}
		(&http::Method::POST, &["repos", repo_id, ""]) => {
			tangram_app_pages_repos_repo_id_index::post(
				context,
				request,
				repo_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", repo_id, "models", "new"]) => {
			tangram_app_pages_repos_repo_id_models_new::get(
				context,
				request,
				repo_id,
			).boxed()
		}
		(&http::Method::POST, &["repos", repo_id, "models", "new"]) => {
			tangram_app_pages_repos_repo_id_models_new::post(
				context,
				request,
				repo_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_index::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id]) => {
			tangram_app_layouts::model_layout::post(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			tangram_app_layouts::model_layout::download(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_grid", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_grid_index::get(
				context, request, model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_stats_index::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_stats_columns_column_name::get(
				context,
				request,
				model_id,
				column_name,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_importances"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_importances::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "prediction"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_prediction::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_metrics_index::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_metrics_class_metrics::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "precision_recall"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_metrics_precision_recall::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_training_metrics_roc::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_tuning::get(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_predictions_index::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_predictions_index::post(
				context,
				request,
				model_id,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", "predictions", identifier]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_predictions_predictions_identifier::get(
				context,
				request,
				model_id,
				identifier,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_stats_index::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", "columns", column_name]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_stats_columns_column_name::get(
				context,
				request,
				model_id,
				column_name,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_metrics_index::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", "class_metrics"]) => {
			tangram_app_pages_repos_repo_id_models_model_id_production_metrics_class_metrics::get(
				context,
				request,
				model_id,
				search_params,
			).boxed()
		}
		(&http::Method::GET, &["user"]) =>{
			tangram_app_pages_user::get(
				context,
				request,
			).boxed()
		},
		(&http::Method::POST, &["user"]) => {
			tangram_app_pages_user::post(
				context,
				request,
			).boxed()
		},
		(&http::Method::GET, &["organizations", "new"]) => {
			tangram_app_pages_organizations_new::get(
				context,
				request,
			).boxed()
		},
		(&http::Method::POST, &["organizations", "new"]) => {
			tangram_app_pages_organizations_new::post(
				context,
				request,
			).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, ""]) => {
			tangram_app_pages_organizations_organization_id_index::get(
				context,
				request,
				organization_id,
			).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, ""]) => {
			tangram_app_pages_organizations_organization_id_index::post(
				context,
				request,
				organization_id,
			).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, "edit"]) => {
			tangram_app_pages_organizations_organization_id_edit::get(
				context,
				request,
				organization_id,
			).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, "members", "new"]) => {
			tangram_app_pages_organizations_organization_id_members_new::get(
				context,
				request,
				organization_id,
			).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, "members", "new"]) => {
			tangram_app_pages_organizations_organization_id_members_new::post(
				context,
				request,
				organization_id,
			).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, "edit"]) => {
			tangram_app_pages_organizations_organization_id_edit::post(
				context,
				request,
				organization_id,
			).boxed()
		}
		_ => async {
			#[cfg(debug_assertions)]
			{
				let out_dir = PathBuf::from(env!("OUT_DIR"));
				if let Some(response) = serve_from_dir(&out_dir, path).await? {
					return Ok(response)
				}
				if let Some(path) = path.strip_prefix("/assets") {
					if let Some(response) = serve_from_dir(&Path::new("."), path).await? {
						return Ok(response)
					}
				}
			}
			#[cfg(not(debug_assertions))]
			{
				use tangram_deps::include_out_dir;
				let dir = include_out_dir::include_out_dir!();
				if let Some(response) = serve_from_include_dir(&dir, path).await? {
					return Ok(response)
				}
			}
			Ok(http::Response::builder()
				.status(http::StatusCode::NOT_FOUND)
				.body(hyper::Body::from("not found"))
				.unwrap())
		}.boxed(),
	};
	let result = result.await;
	let response = match result {
		Ok(response) => response,
		Err(error) => {
			eprintln!("{}", error);
			let body = if cfg!(debug_assertions) {
				format!("{}", error)
			} else {
				"internal server error".to_owned()
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

async fn serve_from_dir(dir: &Path, path: &str) -> Result<Option<http::Response<hyper::Body>>> {
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

#[cfg(not(debug_assertions))]
async fn serve_from_include_dir(
	dir: &tangram_deps::include_out_dir::Dir,
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
