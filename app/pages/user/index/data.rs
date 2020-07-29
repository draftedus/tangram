use crate::app::{helpers, user::User};
use anyhow::Result;
use serde::Serialize;
use tangram::id::Id;
use tokio_postgres as postgres;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOverviewViewModel {
	email: String,
	organizations: Vec<helpers::organizations::Organization>,
	repos: Vec<Repo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
	main_model_id: String,
}

pub async fn data(db: &postgres::Transaction<'_>, user: User) -> Result<UserOverviewViewModel> {
	let organizations = helpers::organizations::get_organizations(&db, user.id).await?;
	let repos = get_user_repositories(&db, user.id).await?;
	Ok(UserOverviewViewModel {
		email: user.email,
		organizations,
		repos,
	})
}

pub async fn get_user_repositories(
	db: &postgres::Transaction<'_>,
	user_id: Id,
) -> Result<Vec<Repo>> {
	let rows = db
		.query(
			"
				select
					repos.id,
					repos.title,
					models.id
				from repos
				join models
					on models.repo_id = repos.id
					and models.is_main = 'true'
				where repos.user_id = $1
      ",
			&[&user_id],
		)
		.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let title: String = row.get(1);
			let main_model_id: Id = row.get(2);
			Repo {
				id: id.to_string(),
				title,
				main_model_id: main_model_id.to_string(),
			}
		})
		.collect())
}
