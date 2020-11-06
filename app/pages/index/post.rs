use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
		user::{authorize_user, authorize_user_for_repo},
	},
	Context,
};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_repo")]
	DeleteRepo(DeleteRepoAction),
}

#[derive(serde::Deserialize)]
struct DeleteRepoAction {
	repo_id: String,
}

pub async fn post(context: &Context, mut request: Request<Body>) -> Result<Response<Body>> {
	let data = match to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	match action {
		Action::DeleteRepo(DeleteRepoAction { repo_id, .. }) => {
			let repo_id: Id = match repo_id.parse() {
				Ok(repo_id) => repo_id,
				Err(_) => return Ok(not_found()),
			};
			if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
				return Ok(not_found());
			}
			sqlx::query(
				"
					delete from repos
					where id = $1
				",
			)
			.bind(&repo_id.to_string())
			.execute(&mut *db)
			.await?;
		}
	}
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
