use crate::{error::Error, user::authorize_user, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tangram_core::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReposNewProps {
	organizations: Vec<Organization>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Organization {
	id: String,
	name: String,
}

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
	let props = ReposNewProps {
		organizations: items,
	};
	let html = context.pinwheel.render("/repos/new", props).await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}
