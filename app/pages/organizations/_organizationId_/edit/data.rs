use crate::app::{
	error::Error,
	helpers::organizations,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use serde::Serialize;
use std::sync::Arc;
use tangram::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationEditViewModel {
	id: String,
	members: Vec<organizations::Member>,
	name: String,
	plan: organizations::Plan,
}

pub async fn data(
	request: Request<Body>,
	context: Arc<Context>,
	organization_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	authorize_user_for_organization(&db, &user, organization_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let organization = organizations::get_organization(organization_id, &db)
		.await?
		.ok_or(Error::NotFound)?;
	let response = OrganizationEditViewModel {
		id: organization_id.to_string(),
		name: organization.name,
		plan: organization.plan,
		members: organization.members,
	};

	let response = serde_json::to_vec(&response)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}
