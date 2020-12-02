use super::render::{render, Props};
use pinwheel::client;
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::{app_layout::get_app_layout_info, document::PageInfo};
use tangram_deps::{http, hyper};
use tangram_util::{error::Result, id::Id};

pub async fn get(
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
	let props = Props {
		app_layout_info,
		error: None,
	};
	let client_wasm_js_src = client!();
	let page_info = PageInfo {
		client_wasm_js_src: Some(client_wasm_js_src),
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	db.commit().await?;
	Ok(response)
}
