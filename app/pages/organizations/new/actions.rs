use crate::app::{
	error::Error,
	user::{authorize_user, User},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram::id::Id;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Action {
	pub name: String,
}

pub async fn actions(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
	create_organization(action, user, db).await
}

async fn create_organization(
	action: Action,
	user: User,
	db: deadpool_postgres::Transaction<'_>,
) -> Result<Response<Body>> {
	let Action { name } = action;
	let created_at: DateTime<Utc> = Utc::now();
	let organization_id: Id = db
		.query_one(
			"
				insert into organizations
					(id, name, created_at, plan)
				values
					($1, $2, $3, 'trial')
				returning id
			",
			&[&Id::new(), &name, &created_at],
		)
		.await?
		.get(0);
	db.execute(
		"
			insert into organizations_users
				(organization_id, user_id, is_admin)
			values
				($1, $2, true)
		",
		&[&organization_id, &user.id],
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
