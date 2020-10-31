use super::props::{Model, Props};
use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_repo},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use chrono::prelude::*;
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
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
	authorize_user_for_repo(&mut db, &user, repo_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let app_layout_info = get_app_layout_info(context).await?;
	let rows = sqlx::query(
		"
			select
				models.id,
				models.created_at
			from models
			where models.repo_id = ?1
			order by models.created_at
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(&mut db)
	.await?;
	let models = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id: Id = id.parse().unwrap();
			let created_at: i64 = row.get(1);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			Model {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
			}
		})
		.collect();
	let props = Props {
		app_layout_info,
		models,
	};
	db.commit().await?;
	let html = context.pinwheel.render_with("/repos/_repo_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
