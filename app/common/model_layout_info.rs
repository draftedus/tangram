use anyhow::Result;
use sqlx::prelude::*;
use tangram_id::Id;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutInfo {
	pub id: String,
	pub title: String,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub model_id: Id,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
	pub name: String,
	pub url: String,
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
	let model_version_ids = super::repos::get_model_version_ids(&mut db, id).await?;
	let organization_id: Option<String> = row.get(2);
	let organization_name: Option<String> = row.get(3);
	let user_id: Option<String> = row.get(4);
	let user_email: Option<String> = row.get(5);
	let owner = match (organization_id, user_id) {
		(Some(organization_id), None) => Some(Owner {
			name: organization_name.unwrap(),
			url: format!("/organizations/{}/", organization_id),
		}),
		(None, Some(_)) => Some(Owner {
			name: user_email.unwrap(),
			url: "/user".to_string(),
		}),
		(None, None) => None,
		(_, _) => unreachable!(),
	};
	Ok(ModelLayoutInfo {
		id: id.to_string(),
		title,
		model_version_ids,
		owner,
		model_id,
	})
}
