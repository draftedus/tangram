use crate::{
	common::user::{authorize_user, authorize_user_for_repo, User},
	error::Error,
	Context,
};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_core::id::Id;

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&mut db, &user).await?;
	db.commit().await?;
	let html = context.pinwheel.render_with("/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

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
}

pub async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &Option<User>,
) -> Result<Props> {
	if let Some(user) = user {
		props_user(db, user).await
	} else {
		props_root(db).await
	}
}

async fn props_user(db: &mut sqlx::Transaction<'_, sqlx::Any>, user: &User) -> Result<Props> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.created_at
			from repos
			where repos.user_id = ?1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let user_repos: Vec<Repo> = rows
		.into_iter()
		.map(|row| {
			let id = row.get(0);
			let title = row.get(1);
			let created_at = row.get::<i64, _>(2);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			let owner_name = user.email.clone();
			Repo {
				id,
				title,
				created_at: created_at.to_rfc3339(),
				owner_name,
			}
		})
		.collect();
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.created_at,
				organizations.title
			from repos
			left join organizations
				on organizations.id = repos.organization_id
			left join organizations_users
				on organizations_users.organization_id = repos.organization_id
				and organizations_users.user_id = ?1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let org_repos: Vec<Repo> = rows
		.into_iter()
		.map(|row: sqlx::any::AnyRow| {
			let id = row.get(0);
			let title = row.get(1);
			let created_at = row.get::<i64, _>(2);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			let org_title = row.get(3);
			Repo {
				id,
				title,
				created_at: created_at.to_rfc3339(),
				owner_name: org_title,
			}
		})
		.collect();
	let mut repos = user_repos;
	repos.extend(org_repos);
	Ok(Props { repos })
}

async fn props_root(db: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<Props> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.created_at,
				repos.title,
				'root' as owner_name
			from repos
			where repos.user_id is null and repos.organization_id is null
			order by repos.created_at
		",
	)
	.fetch_all(db)
	.await?;
	let repos = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id: Id = id.parse().unwrap();
			let created_at: i64 = row.get(1);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			let title = row.get(2);
			let owner_name = row.get(3);
			Repo {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
				owner_name,
				title,
			}
		})
		.collect();
	Ok(Props { repos })
}

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_repo")]
	DeleteRepo(DeleteRepoAction),
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteRepoAction {
	pub repo_id: String,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
	match action {
		Action::DeleteRepo(DeleteRepoAction { repo_id, .. }) => {
			let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
			if let Some(user) = user {
				authorize_user_for_repo(&mut db, &user, repo_id)
					.await
					.map_err(|_| Error::NotFound)?;
			}
			sqlx::query(
				"
					delete from repos
					where id = ?1
				",
			)
			.bind(&repo_id.to_string())
			.execute(&mut *db)
			.await?;
		}
	}
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
