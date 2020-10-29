use super::props::Props;
use crate::{common::error::Error, layouts::app_layout::get_app_layout_info, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	context: &Context,
	_request: Request<Body>,
	_organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let props = props(context, None).await?;
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/members/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn props(context: &Context, error: Option<String>) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	Ok(Props {
		app_layout_info,
		error,
	})
}
