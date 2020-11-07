use super::props::Props;
use crate::{
	common::{
		error::{redirect_to_login, service_unavailable},
		repos::{repos_for_root, repos_for_user},
		timezone::get_timezone,
		user::{authorize_user, User},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
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
	let app_layout_info = get_app_layout_info(context).await?;
	let repos = match user {
		User::Root => repos_for_root(&mut db, &timezone).await?,
		User::Normal(user) => repos_for_user(&mut db, &timezone, &user).await?,
	};
	let props = Props {
		app_layout_info,
		repos,
	};
	db.commit().await?;
	let html = context.pinwheel.render_with("/", props)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
