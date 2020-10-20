use super::render::render;
use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_repo},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response};
use tangram_util::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	repo_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let response = render(context, None).await;
	db.commit().await?;
	response
}
