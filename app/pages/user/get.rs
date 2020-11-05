use super::props::{AuthProps, Inner, NoAuthProps, Organization, Props, Repo};
use crate::{
	common::{
		error::Error,
		organizations::get_organizations,
		user::{authorize_user, User},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(context: &Context, request: Request<Body>) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let app_layout_info = get_app_layout_info(context).await?;
	let props = match user {
		User::Root => {
			let repos = get_root_user_repositories(&mut db).await?;
			Props {
				app_layout_info,
				inner: Inner::NoAuth(NoAuthProps { repos }),
			}
		}
		User::Normal(user) => {
			let organizations = get_organizations(&mut db, user.id)
				.await?
				.into_iter()
				.map(|organization| Organization {
					id: organization.id,
					name: organization.name,
				})
				.collect();
			let repos = get_user_repositories(&mut db, user.id).await?;
			Props {
				app_layout_info,
				inner: Inner::Auth(AuthProps {
					email: user.email,
					organizations,
					repos,
				}),
			}
		}
	};
	db.commit().await?;
	let html = context.pinwheel.render_with("/user", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

async fn get_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.user_id = $1
		",
	)
	.bind(&user_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect())
}

async fn get_root_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
		",
	)
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect())
}
