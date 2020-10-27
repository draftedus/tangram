use crate::{
	common::{
		error::Error,
		user::{authorize_normal_user, NormalUser},
	},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::id::Id;

#[derive(serde::Deserialize, Clone, Debug)]
struct Action {
	name: String,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::BadRequest.into());
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
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let response = create_organization(action, &user, &mut db).await?;
	db.commit().await?;
	Ok(response)
}

async fn create_organization(
	action: Action,
	user: &NormalUser,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Response<Body>> {
	let Action { name } = action;
	let now = Utc::now().timestamp();
	let plan = "trial";
	let organization_id: Id = Id::new();
	sqlx::query(
		"
			insert into organizations
				(id, name, created_at, plan)
			values
				(?1, ?2, ?3, 'trial')
			",
	)
	.bind(&organization_id.to_string())
	.bind(&name)
	.bind(&now)
	.bind(&plan)
	.execute(&mut *db)
	.await?;
	sqlx::query(
		"
			insert into organizations_users
				(organization_id, user_id, is_admin)
			values
				(?1, ?2, 1)
		",
	)
	.bind(&organization_id.to_string())
	.bind(&user.id.to_string())
	.execute(&mut *db)
	.await?;
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
