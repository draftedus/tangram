use super::props::{Props, Repo};
use crate::{
	common::{
		error::{redirect_to_login, service_unavailable},
		timezone::get_timezone,
		user::{authorize_user, NormalUser, User},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use chrono::prelude::*;
use chrono_tz::Tz;
use sqlx::prelude::*;
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let timezone = get_timezone(&request);
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_info = get_app_layout_info(context).await?;
	let repos = match user {
		User::Root => repos_for_root(&mut db, &timezone).await?,
		User::Normal(user) => repos_for_user(&mut db, &timezone, &user).await?,
	};
	let props = Props {
		app_layout_info,
		repos,
	};
	db.commit().await?;
	let html = context.pinwheel.render_with("/", props)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

async fn repos_for_root(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	timezone: &Tz,
) -> Result<Vec<Repo>> {
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
			let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(&timezone);
			let title = row.get(2);
			Repo {
				created_at: created_at.to_string(),
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
	timezone: &Tz,
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
			where repos.user_id = $1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(&mut *db)
	.await?;
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let created_at = row.get::<i64, _>(2);
		let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(timezone);
		let owner_name = user.email.clone();
		repos.push(Repo {
			id,
			title,
			created_at: created_at.to_string(),
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
				and organizations_users.user_id = $1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(&mut *db)
	.await?;
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let created_at = row.get::<i64, _>(2);
		let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(timezone);
		let owner_name = row.get(3);
		repos.push(Repo {
			id,
			title,
			created_at: created_at.to_string(),
			owner_name,
		});
	}
	Ok(repos)
}
