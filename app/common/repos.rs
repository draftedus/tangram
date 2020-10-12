use anyhow::Result;
use chrono::prelude::*;
use sqlx::prelude::*;
use tangram_util::id::Id;

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
				?, ?, ?
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
				?, ?, ?, ?
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
				?, ?, ?, ?
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
				?, ?, ?, ?
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
			repos.id = ?1
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(&mut *db)
	.await?
	.iter()
	.map(|row| row.get::<String, _>(0).parse().unwrap())
	.collect())
}
