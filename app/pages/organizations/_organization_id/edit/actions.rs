use crate::{
	error::Error,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_core::id::Id;

#[derive(serde::Deserialize)]
pub struct Action {
	pub name: String,
}

pub async fn actions(
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
	update_name(action, db, organization_id).await
}

pub async fn update_name(
	action: Action,
	db: deadpool_postgres::Transaction<'_>,
	organization_id: Id,
) -> Result<Response<Body>> {
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
