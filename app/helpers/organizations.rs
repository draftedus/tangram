use anyhow::Result;
use postgres_derive::{FromSql, ToSql};
use tangram::id::Id;
use tokio_postgres as postgres;

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

#[derive(Debug, ToSql, FromSql, serde::Serialize, serde::Deserialize)]
#[postgres(name = "plan")]
pub enum Plan {
	#[postgres(name = "trial")]
	#[serde(rename = "trial")]
	Trial,
	#[postgres(name = "startup")]
	#[serde(rename = "startup")]
	Startup,
	#[postgres(name = "team")]
	#[serde(rename = "team")]
	Team,
	#[postgres(name = "enterprise")]
	#[serde(rename = "enterprise")]
	Enterprise,
}

pub async fn get_organization(
	organization_id: Id,
	db: &deadpool_postgres::Transaction<'_>,
) -> Result<Option<OrganizationResponse>> {
	let rows = db
		.query(
			"
        select
					organizations.name,
					organizations.plan
				from organizations
					where organizations.id = $1
      ",
			&[&organization_id.to_string()],
		)
		.await?;
	// TODO error handling
	let row = if let Some(row) = rows.iter().next() {
		row
	} else {
		return Ok(None);
	};
	let organization_name: String = row.get(0);
	let plan = row.get(1);
	let user_rows = db
		.query(
			"
        select
          users.id,
					users.email,
					organizations_users.is_admin
				from users
				join organizations_users
					on organizations_users.organization_id = $1
					and organizations_users.user_id = users.id
      ",
			&[&organization_id.to_string()],
		)
		.await?;
	let members = user_rows
		.iter()
		.map(|row| {
			let user_id: Id = row.get(0);
			Member {
				id: user_id.to_string(),
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
	db: &postgres::Transaction<'_>,
	user_id: Id,
) -> Result<Vec<Organization>> {
	let rows = db
		.query(
			"
        select
          organizations.id,
          organizations.name
				from organizations
				join organizations_users
					on organizations_users.organization_id = organizations.id
					and organizations_users.user_id = $1
      ",
			&[&user_id],
		)
		.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let name: String = row.get(1);
			Organization {
				id: id.to_string(),
				name,
			}
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
// 	Ok(Response::builder()
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
// 	Ok(Response::builder()
// 		.status(StatusCode::OK)
// 		.header(header::CONTENT_TYPE, "application/json")
// 		.body(Body::from(response))?)
// }
