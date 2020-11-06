use super::props::{Props, Repo};
use crate::{
	common::{
		error::{bad_request, not_found, service_unavailable, unauthorized},
		organizations::get_organization,
		user::{authorize_normal_user, authorize_normal_user_for_organization},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
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
			join models
				on models.repo_id = repos.id
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
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
