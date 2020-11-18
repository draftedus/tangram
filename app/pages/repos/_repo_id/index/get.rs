use super::props::{Model, Props};
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	repos::get_repo,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_info;
use tangram_deps::{
	chrono::prelude::*, chrono_tz::Tz, http, hyper, pinwheel::Pinwheel, sqlx, sqlx::prelude::*,
};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	pinwheel: &Pinwheel,
	context: &Context,
	request: http::Request<hyper::Body>,
	repo_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let timezone = get_timezone(&request);
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	};
	let repo = get_repo(&mut db, &timezone, repo_id).await?;
	let app_layout_info = get_app_layout_info(context).await?;
	let rows = sqlx::query(
		"
			select
				models.id,
				models.created_at
			from models
			where models.repo_id = $1
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
			let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(&timezone);
			Model {
				created_at: created_at.to_string(),
				id: id.to_string(),
			}
		})
		.collect();
	let props = Props {
		app_layout_info,
		models,
		title: repo.title,
	};
	db.commit().await?;
	let html = pinwheel.render_with_props("/repos/_repo_id/", props)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
