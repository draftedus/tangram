use super::render::{render, Props};
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::{app_layout::get_app_layout_info, document::PageInfo};
use tangram_deps::{
	base64, bytes::Buf, chrono::prelude::*, http, hyper, multer, multer::Multipart, sqlx,
};
use tangram_util::{error::Result, id::Id};

pub async fn post(
	context: &Context,
	request: http::Request<hyper::Body>,
	repo_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	}
	let app_layout_info = get_app_layout_info(context).await?;
	let boundary = match request
		.headers()
		.get(http::header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			let props = Props {
				app_layout_info,
				error: Some(format!(
					"Failed to parse request body.\n{}:{}",
					file!(),
					line!()
				)),
			};
			let page_info = PageInfo {
				client_wasm_js_src: None,
			};
			let html = render(props, page_info);
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				let props = Props {
					app_layout_info,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
				};
				let page_info = PageInfo {
					client_wasm_js_src: None,
				};
				let html = render(props, page_info);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		};
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"file" => file = Some(field_data),
			_ => {
				let props = Props {
					app_layout_info,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
				};
				let page_info = PageInfo {
					client_wasm_js_src: None,
				};
				let html = render(props, page_info);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		}
	}
	let file = match file {
		Some(file) => file,
		None => {
			let props = Props {
				app_layout_info,
				error: Some("A file is required.".to_owned()),
			};
			let page_info = PageInfo {
				client_wasm_js_src: None,
			};
			let html = render(props, page_info);
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let model = match tangram_core::model::Model::from_slice(&file) {
		Ok(model) => model,
		Err(_) => {
			let props = Props {
				app_layout_info,
				error: Some("Invalid tangram model file.".to_owned()),
			};
			let page_info = PageInfo {
				client_wasm_js_src: None,
			};
			let html = render(props, page_info);
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let now = Utc::now().timestamp();
	let result = sqlx::query(
		"
			insert into models
				(id, repo_id, created_at, data)
			values
				($1, $2, $3, $4)
		",
	)
	.bind(&model.id().to_string())
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&base64::encode(file))
	.execute(&mut *db)
	.await;
	if result.is_err() {
		let props = Props {
			app_layout_info,
			error: Some("There was an error uploading your model.".to_owned()),
		};
		let page_info = PageInfo {
			client_wasm_js_src: None,
		};
		let html = render(props, page_info);
		let response = http::Response::builder()
			.status(http::StatusCode::BAD_REQUEST)
			.body(hyper::Body::from(html))
			.unwrap();
		return Ok(response);
	};
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/repos/{}/models/{}/", repo_id, model.id()),
		)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
