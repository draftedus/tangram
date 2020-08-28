use crate::{
	common::{
		organizations::{get_organizations, Organization},
		repos::get_user_repos,
		user::{authorize_user, User},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use chrono::Utc;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde::Serialize;

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
	let html = context.pinwheel.render_with("/user", props)?;
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
}

pub async fn props(mut db: &mut sqlx::Transaction<'_, sqlx::Any>, user: User) -> Result<Props> {
	let organizations = get_organizations(&mut db, user.id).await?;
	let repos = get_user_repos(&mut db, user.id)
		.await?
		.into_iter()
		.map(|repo| Repo {
			id: repo.id,
			title: repo.title,
		})
		.collect();
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
