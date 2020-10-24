use crate::Context;
use anyhow::Result;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutInfo {
	pub model_id: Id,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub topbar_avatar: Option<TopbarAvatar>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopbarAvatar {
	avatar_url: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Owner {
	#[serde(rename = "user")]
	User { id: Id, email: String },
	#[serde(rename = "organization")]
	Organization { id: Id, name: String },
}

pub async fn get_model_layout_info(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	model_id: Id,
) -> Result<ModelLayoutInfo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.user_id,
				users.email,
				repos.organization_id,
				organizations.name
			from repos
			join models
				on models.repo_id = repos.id
			left join users
				on users.id = repos.user_id
			left join organizations
				on organizations.id = repos.organization_id
			where models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let repo_id: String = row.get(0);
	let repo_id: Id = repo_id.parse()?;
	let repo_title: String = row.get(1);
	let model_version_ids = super::repos::get_model_version_ids(&mut db, repo_id).await?;
	let owner_organization_id: Option<String> = row.get(2);
	let owner_organization_name: Option<String> = row.get(3);
	let owner_user_id: Option<String> = row.get(4);
	let owner_user_email: Option<String> = row.get(5);
	let owner = if let Some(owner_user_id) = owner_user_id {
		Some(Owner::User {
			id: owner_user_id.parse().unwrap(),
			email: owner_user_email.unwrap(),
		})
	} else if let Some(owner_organization_id) = owner_organization_id {
		Some(Owner::Organization {
			id: owner_organization_id.parse().unwrap(),
			name: owner_organization_name.unwrap(),
		})
	} else {
		None
	};
	let topbar_avatar = if context.options.auth_enabled {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(ModelLayoutInfo {
		model_id,
		model_version_ids,
		owner,
		repo_id: repo_id.to_string(),
		repo_title,
		topbar_avatar,
	})
}
