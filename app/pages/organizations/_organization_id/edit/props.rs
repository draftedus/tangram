use crate::{
	common::{
		error::Error,
		organizations,
		user::{authorize_user, authorize_user_for_organization},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	id: String,
	members: Vec<organizations::Member>,
	name: String,
	plan: organizations::Plan,
}

pub async fn props(
	request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Props> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let user = user.unwrap();
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	authorize_user_for_organization(&mut db, &user, organization_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let organization = organizations::get_organization(organization_id, &mut db)
		.await?
		.ok_or(Error::NotFound)?;
	Ok(Props {
		id: organization_id.to_string(),
		name: organization.name,
		plan: organization.plan,
		members: organization.members,
	})
}
