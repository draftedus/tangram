use anyhow::{format_err, Result};
use sqlx::prelude::*;
use tangram::util::id::Id;

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
	// assemble the response
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

// pub async fn get(request: Request<Body>, context: Arc<Context>) -> Result<Response<Body>> {
// 	let mut db = if let Ok(db) = context.database_pool.get().await {
// 		db
// 	} else {
// 		return Err(Error::ServiceUnavailable.into());
// 	};
// 	let db = db.transaction().await?;
// 	let user = if let Ok(user) = authorize_user(&request, &db).await? {
// 		user
// 	} else {
// 		return Err(Error::Unauthorized.into());
// 	};
// 	let organizations = get_organizations(&db, user.id).await?;
// 	db.commit().await?;
// 	let response = OrganizationListResponse { organizations };
// 	let response = serde_json::to_vec(&response)?;
// 	let response = Response::builder()
// 		.status(StatusCode::OK)
// 		.header(header::CONTENT_TYPE, "application/json")
// 		.body(Body::from(response))?)
// }

// pub async fn get(
// 	request: Request<Body>,
// 	context: Arc<Context>,
// 	organization_id: &str,
// ) -> Result<Response<Body>> {
// 	let mut db = if let Ok(db) = context.database_pool.get().await {
// 		db
// 	} else {
// 		return Err(Error::ServiceUnavailable.into());
// 	};
// 	let db = db.transaction().await?;
// 	let user = if let Ok(user) = authorize_user(&request, &db).await? {
// 		user
// 	} else {
// 		return Err(Error::Unauthorized.into());
// 	};
// 	let organization_id: Id = if let Ok(organization_id) = organization_id.parse() {
// 		organization_id
// 	} else {
// 		return Err(Error::NotFound.into());
// 	};
// 	if !authorize_user_for_organization(&db, &user, organization_id).await? {
// 		return Err(Error::NotFound.into());
// 	}
// 	let organization = match get_organization(organization_id, &db).await? {
// 		Some(organization) => organization,
// 		None => return Err(Error::NotFound.into()),
// 	};
// 	let response = serde_json::to_vec(&organization)?;
// 	let response = Response::builder()
// 		.status(StatusCode::OK)
// 		.header(header::CONTENT_TYPE, "application/json")
// 		.body(Body::from(response))?)
// }
