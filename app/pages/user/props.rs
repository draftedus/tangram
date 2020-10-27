use crate::{
	common::{
		organizations::{get_organizations, Organization},
		user::User,
	},
	layouts::app_layout::{get_app_layout_info, AppLayoutInfo},
	Context,
};
use anyhow::Result;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	app_layout_info: AppLayoutInfo,
	inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	#[serde(rename = "auth")]
	Auth(AuthProps),
	#[serde(rename = "no_auth")]
	NoAuth(NoAuthProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProps {
	email: String,
	organizations: Vec<Organization>,
	repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoAuthProps {
	repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
}

pub async fn props(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	user: User,
) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	match user {
		User::Root => {
			let repos = get_root_user_repositories(&mut db).await?;
			Ok(Props {
				app_layout_info,
				inner: Inner::NoAuth(NoAuthProps { repos }),
			})
		}
		User::Normal(user) => {
			let organizations = get_organizations(&mut db, user.id).await?;
			let repos = get_user_repositories(&mut db, user.id).await?;
			Ok(Props {
				app_layout_info,
				inner: Inner::Auth(AuthProps {
					email: user.email,
					organizations,
					repos,
				}),
			})
		}
	}
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
			where repos.user_id = ?1
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
