use super::props::*;
use crate::{
	common::user::User,
	common::{error::Error, user::authorize_user},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;

pub async fn get(context: &Context, request: Request<Body>) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&mut db, context, user, None, None, None).await?;
	let html = context.pinwheel.render_with("/repos/new", props)?;
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	user: User,
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	let owners = match user {
		User::Root => None,
		User::Normal(user) => {
			let mut owners = vec![Owner {
				value: format!("user:{}", user.id),
				title: user.email,
			}];
			let rows = sqlx::query(
				"
				select
					organizations.id,
					organizations.name
				from organizations
				join organizations_users
					on organizations_users.organization_id = organizations.id
					and organizations_users.user_id = ?1
			",
			)
			.bind(&user.id.to_string())
			.fetch_all(&mut *db)
			.await?;
			for row in rows {
				let id: String = row.get(0);
				let title: String = row.get(1);
				owners.push(Owner {
					value: format!("organization:{}", id),
					title,
				})
			}
			Some(owners)
		}
	};
	Ok(Props {
		app_layout_info,
		owners,
		error,
		owner,
		title,
	})
}
