use crate::{
	error::Error,
	helpers,
	user::{authorize_user, User},
	Context,
};
use anyhow::Result;
use chrono::Utc;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;
use tokio_postgres as postgres;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	email: String,
	organizations: Vec<helpers::organizations::Organization>,
	repos: Vec<Repo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
	main_model_id: String,
}

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
	let html = context.pinwheel.render("/user/", props).await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

pub async fn props(db: &mut sqlx::Transaction<'_, sqlx::Any>, user: User) -> Result<Props> {
	let organizations = helpers::organizations::get_organizations(&db, user.id).await?;
	let repos = get_user_repositories(&db, user.id).await?;
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
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let response = match action {
		Action::Logout => logout(user, &db).await?,
	};
	db.commit().await?;
	Ok(response)
}

pub async fn logout(
	user: User,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Response<Body>> {
	let now = Utc::now().timestamp();
	db.execute(
		"
			update
				tokens
			set
				deleted_at = ?1
			where
				token = ?2
		",
		&[&user.token],
	)
	.await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/login")
		.header(header::SET_COOKIE, "auth=; Path=/; Max-Age=0")
		.body(Body::empty())?)
}

pub async fn get_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<Repo>> {
	let rows = db
		.query(
			"
				select
					repos.id,
					repos.title,
					models.id
				from repos
				join models
					on models.repo_id = repos.id
					and models.is_main = 'true'
				where repos.user_id = $1
      ",
			&[&user_id],
		)
		.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let title: String = row.get(1);
			let main_model_id: Id = row.get(2);
			Repo {
				id: id.to_string(),
				title,
				main_model_id: main_model_id.to_string(),
			}
		})
		.collect())
}
