use super::props::{Props, Repo};
use crate::{
	common::{
		error::Error,
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
		return Err(Error::NotFound.into());
	}
	let app_layout_info = get_app_layout_info(context).await?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
	let organization = get_organization(organization_id, &mut db)
		.await?
		.ok_or(Error::NotFound)?;
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
