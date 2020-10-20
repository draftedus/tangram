use crate::common::{
	organizations::{get_organizations, Organization},
	user::User,
};
use anyhow::Result;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
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
	user: Option<User>,
) -> Result<Props> {
	if let Some(user) = user {
		let organizations = get_organizations(&mut db, user.id).await?;
		let repos = get_user_repositories(&mut db, user.id).await?;
		Ok(Props {
			inner: Inner::Auth(AuthProps {
				email: user.email,
				organizations,
				repos,
			}),
		})
	} else {
		let repos = get_root_user_repositories(&mut db).await?;
		Ok(Props {
			inner: Inner::NoAuth(NoAuthProps { repos }),
		})
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
