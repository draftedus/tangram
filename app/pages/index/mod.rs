use crate::user::User;
use crate::{error::Error, user::authorize_user, Context};
use anyhow::Result;
use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use tangram_core::id::Id;
use tokio_postgres as postgres;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub repos: Vec<Repo>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
	pub created_at: String,
	pub owner_name: String,
	pub main_model_id: String,
}

pub async fn props(db: &postgres::Transaction<'_>, user: &User) -> Result<Props> {
	let rows = db
		.query(
			"
				select
					repos.id,
					repos.created_at,
					repos.title,
					case
						when repos.organization_id is null
							then users.email
						when repos.user_id is null
							then organizations.name
					end as owner_name,
					models.id
				from repos
				left join organizations
					on organizations.id = repos.organization_id
				left join organizations_users
					on organizations_users.organization_id = repos.organization_id
					and organizations_users.user_id = $1
				left join users
					on users.id = repos.user_id
					and users.id = $1
				join models
					on models.repo_id = repos.id
					and models.is_main = 'true'
				where organizations_users.user_id = $1 or users.id = $1
				order by repos.created_at
			",
			&[&user.id],
		)
		.await?;
	let repos = rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let created_at: DateTime<Utc> = row.get(1);
			let title = row.get(2);
			let owner_name = row.get(3);
			let main_model_id: Id = row.get(4);
			Repo {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
				owner_name,
				title,
				main_model_id: main_model_id.to_string(),
			}
		})
		.collect();
	Ok(Props { repos })
}

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&db, &user).await?;
	db.commit().await?;
	let html = context.pinwheel.render("/", props).await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

pub async fn post(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.body(Body::empty())?)
}
