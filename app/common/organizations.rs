use anyhow::{format_err, Result};
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationListResponse {
	pub organizations: Vec<Organization>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
	pub id: String,
	pub name: String,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationResponse {
	pub id: String,
	pub name: String,
	pub members: Vec<Member>,
	pub plan: Plan,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Member {
	pub id: String,
	pub email: String,
	pub is_admin: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Plan {
	#[serde(rename = "trial")]
	Trial,
	#[serde(rename = "startup")]
	Startup,
	#[serde(rename = "team")]
	Team,
	#[serde(rename = "enterprise")]
	Enterprise,
}

pub async fn get_organization(
	organization_id: Id,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Option<OrganizationResponse>> {
	let row = sqlx::query(
		"
			select
				organizations.name,
				organizations.plan
			from organizations
				where organizations.id = ?1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let organization_name: String = row.get(0);
	let plan: String = row.get(1);
	let plan = match plan.as_str() {
		"trial" => Plan::Trial,
		"team" => Plan::Team,
		"startup" => Plan::Startup,
		"enterprise" => Plan::Enterprise,
		_ => return Err(format_err!("bad plan {}", plan)),
	};
	let user_rows = sqlx::query(
		"
			select
				users.id,
				users.email,
				organizations_users.is_admin
			from users
			join organizations_users
				on organizations_users.organization_id = ?1
				and organizations_users.user_id = users.id
		",
	)
	.bind(&organization_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let members = user_rows
		.iter()
		.map(|row| {
			let user_id: String = row.get(0);
			Member {
				id: user_id,
				email: row.get(1),
				is_admin: row.get(2),
			}
		})
		.collect();
	Ok(Some(OrganizationResponse {
		id: organization_id.to_string(),
		members,
		name: organization_name,
		plan,
	}))
}

pub async fn get_organizations(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<Organization>> {
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
	.bind(&user_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let name: String = row.get(1);
			Organization { id, name }
		})
		.collect())
}
