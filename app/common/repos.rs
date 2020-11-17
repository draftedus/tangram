use crate::user::NormalUser;
use tangram_deps::{base64, chrono::prelude::*, chrono_tz::Tz, sqlx, sqlx::prelude::*};
use tangram_util::{error::Result, id::Id};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
	pub created_at: String,
	pub owner_name: Option<String>,
}

pub async fn get_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	timezone: &Tz,
	id: Id,
) -> Result<Repo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.created_at,
				repos.title
			from repos
			where repos.id = $1
		",
	)
	.bind(&id.to_string())
	.fetch_one(db)
	.await?;
	let id: String = row.get(0);
	let id: Id = id.parse().unwrap();
	let created_at: i64 = row.get(1);
	let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(&timezone);
	let title = row.get(2);
	let repo = Repo {
		created_at: created_at.to_string(),
		id: id.to_string(),
		owner_name: None,
		title,
	};
	Ok(repo)
}

pub async fn repos_for_root(
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

pub async fn repos_for_user(
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

pub async fn create_root_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, title, created_at
			) values (
				$1, $2, $3
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&title)
	.bind(&Utc::now().timestamp())
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn create_user_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, title, user_id, created_at
			) values (
				$1, $2, $3, $4
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&title)
	.bind(&user_id.to_string())
	.bind(&Utc::now().timestamp())
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn create_org_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	org_id: Id,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, title, organization_id, created_at
			) values (
				$1, $2, $3, $4
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&title)
	.bind(&org_id.to_string())
	.bind(&Utc::now().timestamp())
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn add_model_version(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
	model_id: Id,
	model_data: &[u8],
) -> Result<()> {
	sqlx::query(
		"
			insert into models
				(id, repo_id, data, created_at)
			values (
				$1, $2, $3, $4
			)
		",
	)
	.bind(&model_id.to_string())
	.bind(&repo_id.to_string())
	.bind(&base64::encode(&model_data))
	.bind(&Utc::now().timestamp())
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn get_model_version_ids(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
) -> Result<Vec<Id>> {
	Ok(sqlx::query(
		"
			select
				models.id
			from models
			join repos
				on models.repo_id = repos.id
			where
			repos.id = $1
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(&mut *db)
	.await?
	.iter()
	.map(|row| row.get::<String, _>(0).parse().unwrap())
	.collect())
}
