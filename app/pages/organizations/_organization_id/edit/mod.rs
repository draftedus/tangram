use crate::{
	error::Error,
	helpers::organizations,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, organization_id).await?;
	let html = context
		.pinwheel
		.render("/organizations/_organization_id/edit", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	members: Vec<organizations::Member>,
	name: String,
	plan: organizations::Plan,
}

async fn props(request: Request<Body>, context: &Context, organization_id: &str) -> Result<Props> {
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
	Ok(Props {
		id: organization_id.to_string(),
		name: organization.name,
		plan: organization.plan,
		members: organization.members,
	})
}

#[derive(serde::Deserialize)]
pub struct Action {
	pub name: String,
}

pub async fn post(
	mut request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
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

	let Action { name } = action;

	db.execute(
		"
			update organizations
				set name = $1
			where organizations.id = $2
		",
		&[&name, &organization_id],
	)
	.await?;
	db.commit().await?;

	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())?)
}
