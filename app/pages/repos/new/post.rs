use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_organization},
	},
	Context,
};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Action {
	title: String,
	owner: Option<String>,
}

pub async fn post(context: &Context, mut request: Request<Body>) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let Action { title, owner } = action;
	let repo_id = Id::new();
	if let Some(owner) = &owner {
		let owner_parts: Vec<&str> = owner.split(':').collect();
		let owner_type = owner_parts.get(0).ok_or(Error::BadRequest)?;
		let owner_id: Id = owner_parts
			.get(1)
			.ok_or(Error::BadRequest)?
			.parse()
			.map_err(|_| Error::BadRequest)?;
		match *owner_type {
			"user" => {
				crate::common::repos::create_user_repo(&mut db, owner_id, repo_id, &title).await?;
			}
			"organization" => {
				if !authorize_user_for_organization(&mut db, &user, owner_id).await? {
					return Err(Error::Unauthorized.into());
				}
				crate::common::repos::create_org_repo(&mut db, owner_id, repo_id, title.as_str())
					.await?;
			}
			_ => return Err(Error::BadRequest.into()),
		}
	} else {
		crate::common::repos::create_root_repo(&mut db, repo_id, title.as_str()).await?;
	};
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, format!("/repos/{}/", repo_id))
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
