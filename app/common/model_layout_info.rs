use crate::Context;
use anyhow::Result;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutInfo {
	pub id: String,
	pub model_id: Id,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub topbar_avatar: Option<TopbarAvatar>,
	pub title: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopbarAvatar {
	avatar_url: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Owner {
	User { id: Id, email: String },
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
	let id: String = row.get(0);
	let id: Id = id.parse()?;
	let title: String = row.get(1);
	let model_version_ids = super::repos::get_model_version_ids(&mut db, id).await?;
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
		id: id.to_string(),
		model_id,
		model_version_ids,
		owner,
		title,
		topbar_avatar,
	})
}
