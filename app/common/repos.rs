use anyhow::Result;
use serde::Serialize;
use sqlx::prelude::*;
use tangram_core::id::Id;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutInfo {
	pub id: String,
	pub title: String,
	pub models: Vec<RepoModel>,
	pub owner_name: String,
	pub owner_url: String,
	pub model_id: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum RepoOwner {
	User(UserOwner),
	Organization(OrganizationOwner),
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserOwner {
	pub email: String,
	pub id: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationOwner {
	pub id: String,
	pub name: String,
}

pub async fn get_model_layout_info(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<ModelLayoutInfo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.organization_id,
				organizations.name,
				repos.user_id,
				users.email
			from repos
			join models
				on models.repo_id = repos.id
			left join organizations
				on organizations.id = repos.organization_id
			left join users
				on users.id = repos.user_id
			where models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let id: String = row.get(0);
	let id: Id = id.parse()?;
	let title: String = row.get(1);
	let models = get_models_for_repo(&mut db, id).await?;
	let organization_id: Option<String> = row.get(2);
	let organization_name: Option<String> = row.get(3);
	let user_id: Option<String> = row.get(4);
	let user_email: Option<String> = row.get(5);
	let owner = match organization_id {
		None => RepoOwner::User(UserOwner {
			email: user_email.unwrap(),
			id: user_id.unwrap(),
		}),
		Some(organization_id) => RepoOwner::Organization(OrganizationOwner {
			id: organization_id,
			name: organization_name.unwrap(),
		}),
	};
	let (owner_name, owner_url) = match owner {
		RepoOwner::User(user) => {
			let owner_url = "/user".to_string();
			(user.email, owner_url)
		}
		RepoOwner::Organization(organization) => {
			let owner_url = format!("/organizations/{}/", organization.id);
			(organization.name, owner_url)
		}
	};
	let RepoModel { id: model_id, .. } = models
		.iter()
		.find(|model| model.id == model_id.to_string())
		.unwrap();
	let model_id = model_id.clone();
	Ok(ModelLayoutInfo {
		id: id.to_string(),
		title,
		models,
		owner_name,
		owner_url,
		model_id,
	})
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepoModel {
	pub id: String,
}

async fn get_models_for_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
) -> Result<Vec<RepoModel>> {
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
	.map(|row| {
		let id: String = row.get(0);
		RepoModel { id }
	})
	.collect())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
}

pub async fn get_user_repos(
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
	let rows = rows
		.into_iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect();
	Ok(rows)
}

pub async fn get_organization_repos(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			join models
				on models.repo_id = repos.id
			where repos.organization_id = ?1
		",
	)
	.bind(&organization_id.to_string())
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
