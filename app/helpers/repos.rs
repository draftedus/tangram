use crate::types;
use anyhow::Result;
use serde::Serialize;
use sqlx::prelude::*;
use tangram_core::id::Id;

pub async fn get_model_layout_props(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<types::ModelLayoutProps> {
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
		Some(organization_id) => types::RepoOwner::Organization(types::OrganizationOwner {
			id: organization_id,
			name: organization_name.unwrap(),
		}),
		None => types::RepoOwner::User(types::UserOwner {
			email: user_email.unwrap(),
			id: user_id.unwrap(),
		}),
	};

	let (owner_name, owner_url) = match owner {
		types::RepoOwner::Organization(organization) => {
			let owner_url = format!("/organizations/{}/", organization.id);
			(organization.name, owner_url)
		}
		types::RepoOwner::User(user) => {
			let owner_url = "/user/".to_string();
			(user.email, owner_url)
		}
	};

	let types::RepoModel {
		id: model_id,
		title: model_title,
		..
	} = models
		.iter()
		.find(|model| model.id == model_id.to_string())
		.unwrap();

	let model_id = model_id.clone();
	let model_title = model_title.clone();

	Ok(types::ModelLayoutProps {
		id: id.to_string(),
		title,
		models,
		owner_name,
		owner_url,
		model_id,
		model_title,
	})
}

async fn get_models_for_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
) -> Result<Vec<types::RepoModel>> {
	Ok(sqlx::query(
		"
			select
				models.id,
				models.title,
				models.is_main
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
		let title: String = row.get(1);
		let is_main: bool = row.get(2);
		types::RepoModel { id, title, is_main }
	})
	.collect())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
	pub main_model_id: String,
}

pub async fn get_organization_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
				select
					repos.id,
					repos.title,
					models.id
				from repos
				join models
					on models.repo_id = repos.id
					and models.is_main = 'true'
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
			let main_model_id: String = row.get(2);
			Repo {
				id,
				title,
				main_model_id,
			}
		})
		.collect())
}
