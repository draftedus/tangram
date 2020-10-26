use crate::{
	common::user::User,
	layouts::app_layout::{get_app_layout_info, AppLayoutInfo},
	Context,
};
use anyhow::Result;
use sqlx::prelude::*;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
	pub title: Option<String>,
	pub owner: Option<String>,
	pub owners: Option<Vec<Owner>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
	pub value: String,
	pub title: String,
}

pub async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	user: Option<User>,
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	let owners = if let Some(user) = user {
		let mut owners = vec![Owner {
			value: format!("user:{}", user.id),
			title: user.email,
		}];
		let rows = sqlx::query(
			"
				select
					organizations.id,
					organizations.name
				from organizations
				join organizations_users
					on organizations_users.organization_id = organizations.id
					and organizations_users.user_id = ?1
			",
		)
		.bind(&user.id.to_string())
		.fetch_all(&mut *db)
		.await?;
		for row in rows {
			let id: String = row.get(0);
			let title: String = row.get(1);
			owners.push(Owner {
				value: format!("organization:{}", id),
				title,
			})
		}
		Some(owners)
	} else {
		None
	};
	Ok(Props {
		app_layout_info,
		owners,
		error,
		owner,
		title,
	})
}
