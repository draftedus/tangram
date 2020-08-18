use crate::{
	error::Error,
	helpers::organizations::{get_organizations, Organization},
	user::{authorize_user, User},
	Context,
};
use anyhow::Result;
use chrono::Utc;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde::Serialize;
use sqlx::prelude::*;
use tangram_core::id::Id;

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&mut db, user).await?;
	db.commit().await?;
	let html = context.pinwheel.render_with("/user/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	email: String,
	organizations: Vec<Organization>,
	repos: Vec<Repo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
	main_model_id: String,
}

pub async fn props(mut db: &mut sqlx::Transaction<'_, sqlx::Any>, user: User) -> Result<Props> {
	let organizations = get_organizations(&mut db, user.id).await?;
	let repos = get_user_repositories(&mut db, user.id).await?;
	Ok(Props {
		email: user.email,
		organizations,
		repos,
	})
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Action {
	Logout,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let response = match action {
		Action::Logout => logout(user, &mut db).await?,
	};
	db.commit().await?;
	Ok(response)
}

pub async fn logout(
	user: User,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Response<Body>> {
	let now = Utc::now().timestamp();
	sqlx::query(
		"
			update
				tokens
			set
				deleted_at = ?1
			where
				token = ?2
		",
	)
	.bind(&now)
	.bind(&user.token)
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/login")
		.header(header::SET_COOKIE, "auth=; Path=/; Max-Age=0")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

pub async fn get_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				models.id
			from repos
			join models
				on models.repo_id = repos.id
				and models.is_main = 1
			where repos.user_id = ?1
		",
	)
	.bind(&user_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			let main_model_id: String = row.get(2);
			Repo {
				id,
				title,
				main_model_id,
			}
		})
		.collect())
}
