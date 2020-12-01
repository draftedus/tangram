use super::page::{render, Props, Repo};
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	organizations::get_organization,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_app_layouts::{app_layout::get_app_layout_info, document::PageInfo};
use tangram_deps::{http, hyper, sqlx, sqlx::prelude::*};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	organization_id: &str,
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
	let app_layout_info = get_app_layout_info(context).await?;
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(&request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	}
	let organization = match get_organization(organization_id, &mut db).await? {
		Some(organization) => organization,
		None => return Ok(not_found()),
	};
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.organization_id = $1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let repos = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect();
	let props = Props {
		app_layout_info,
		id: organization_id.to_string(),
		members: organization.members,
		name: organization.name,
		repos,
		user_id: user.id.to_string(),
	};
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
