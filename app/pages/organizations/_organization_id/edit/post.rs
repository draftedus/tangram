use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_organization},
	},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::id::Id;

#[derive(serde::Deserialize)]
struct Action {
	name: String,
}

pub async fn post(
	context: &Context,
	mut request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	authorize_user_for_organization(&mut db, &user, organization_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let Action { name } = action;
	sqlx::query(
		"
			update organizations
				set name = ?1
			where organizations.id = ?2
		",
	)
	.bind(&name)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
