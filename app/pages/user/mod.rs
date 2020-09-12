use crate::app::{
	common::{
		organizations::{get_organizations, Organization},
		user::{authorize_user, User},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use chrono::Utc;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram::util::id::Id;

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	#[serde(rename = "auth")]
	Auth(AuthProps),
	#[serde(rename = "no_auth")]
	NoAuth(NoAuthProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProps {
	email: String,
	organizations: Vec<Organization>,
	repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoAuthProps {
	repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
}

pub async fn props(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: Option<User>,
) -> Result<Props> {
	if let Some(user) = user {
		let organizations = get_organizations(&mut db, user.id).await?;
		let repos = get_user_repositories(&mut db, user.id).await?;
		Ok(Props {
			inner: Inner::Auth(AuthProps {
				email: user.email,
				organizations,
				repos,
			}),
		})
	} else {
		let repos = get_root_user_repositories(&mut db).await?;
		Ok(Props {
			inner: Inner::NoAuth(NoAuthProps { repos }),
		})
	}
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Action {
	Logout,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
				repos.title
			from repos
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
			Repo { id, title }
		})
		.collect())
}

pub async fn get_root_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
		",
	)
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect())
}
