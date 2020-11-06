use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
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
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let data = match to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let Action { title, owner } = action;
	let repo_id = Id::new();
	if let Some(owner) = &owner {
		let owner_parts: Vec<&str> = owner.split(':').collect();
		let owner_type = match owner_parts.get(0) {
			Some(owner_type) => owner_type,
			None => return Ok(bad_request()),
		};
		let owner_id = match owner_parts.get(1) {
			Some(owner_id) => owner_id,
			None => return Ok(bad_request()),
		};
		let owner_id: Id = match owner_id.parse() {
			Ok(owner_id) => owner_id,
			Err(_) => return Ok(bad_request()),
		};
		match *owner_type {
			"user" => {
				crate::common::repos::create_user_repo(&mut db, owner_id, repo_id, &title).await?;
			}
			"organization" => {
				if !authorize_user_for_organization(&mut db, &user, owner_id).await? {
					return Ok(not_found());
				};
				crate::common::repos::create_org_repo(&mut db, owner_id, repo_id, title.as_str())
					.await?;
			}
			_ => return Ok(bad_request()),
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
