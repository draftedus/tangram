use super::props::{Props, Repo};
use crate::{
	common::{
		error::Error,
		user::{authorize_user, NormalUser, User},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use chrono::prelude::*;
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(context: &Context, request: Request<Body>) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let app_layout_info = get_app_layout_info(context).await?;
	let repos = match user {
		User::Root => repos_for_root(&mut db).await?,
		User::Normal(user) => repos_for_user(&mut db, &user).await?,
	};
	let props = Props {
		app_layout_info,
		repos,
	};
	db.commit().await?;
	let html = context.pinwheel.render_with("/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

async fn repos_for_root(db: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.created_at,
				repos.title
			from repos
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
			Repo {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
				owner_name: None,
				title,
			}
		})
		.collect();
	Ok(repos)
}
async fn repos_for_user(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &NormalUser,
) -> Result<Vec<Repo>> {
	let mut repos = Vec::new();
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
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let created_at = row.get::<i64, _>(2);
		let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
		let owner_name = user.email.clone();
		repos.push(Repo {
			id,
			title,
			created_at: created_at.to_rfc3339(),
			owner_name: Some(owner_name),
		});
	}
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.created_at,
				organizations.name
			from repos
			inner join organizations
				on organizations.id = repos.organization_id
			inner join organizations_users
				on organizations_users.organization_id = repos.organization_id
				and organizations_users.user_id = ?1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(&mut *db)
	.await?;
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let created_at = row.get::<i64, _>(2);
		let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
		let owner_name = row.get(3);
		repos.push(Repo {
			id,
			title,
			created_at: created_at.to_rfc3339(),
			owner_name,
		});
	}
	Ok(repos)
}
