use crate::app::{error::Error, user::authorize_user, Context};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tangram::id::Id;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReposNewViewModel {
	organizations: Vec<Organization>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Organization {
	id: String,
	name: String,
}

pub async fn data(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
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
			&[&user.id],
		)
		.await?;
	db.commit().await?;
	let items: Vec<_> = rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let name: String = row.get(1);
			Organization {
				id: id.to_string(),
				name,
			}
		})
		.collect();
	let response = ReposNewViewModel {
		organizations: items,
	};
	let response = serde_json::to_vec(&response)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}
