use crate::{
	common::user::User,
	layouts::app_layout::{get_app_layout_info, AppLayoutInfo},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	app_layout_info: AppLayoutInfo,
	repos: Vec<Repo>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
	created_at: String,
	owner_name: Option<String>,
}

pub async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	user: &Option<User>,
) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	let repos = if let Some(user) = user {
		repos_for_user(db, user).await
	} else {
		repos_for_root(db).await
	}?;
	Ok(Props {
		app_layout_info,
		repos,
	})
}

async fn repos_for_user(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
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
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let created_at = row.get::<i64, _>(2);
		let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
		let org_title = row.get(3);
		repos.push(Repo {
			id,
			title,
			created_at: created_at.to_rfc3339(),
			owner_name: org_title,
		});
	}
	Ok(repos)
}

async fn repos_for_root(db: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.created_at,
				repos.title
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
