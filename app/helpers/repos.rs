use crate::types;
use anyhow::Result;
use tangram_core::id::Id;

pub async fn get_repo_for_model(
	db: &deadpool_postgres::Transaction<'_>,
	model_id: Id,
) -> Result<types::Repo> {
	let row = db
		.query_one(
			"
        select
					repos.id,
					repos.title,
					repos.organization_id,
					organizations.name,
					repos.user_id,
					users.email
        from repos
        join models
					on models.repo_id = repos.id
				left join organizations
					on organizations.id = repos.organization_id
				left join users
					on users.id = repos.user_id
        where models.id = $1
      ",
			&[&model_id],
		)
		.await?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let models = get_models_for_repo(&db, id).await?;
	let organization_id: Option<Id> = row.get(2);
	let organization_name: Option<String> = row.get(3);
	let user_id: Option<Id> = row.get(4);
	let user_email: Option<String> = row.get(5);
	let owner = match organization_id {
		Some(organization_id) => types::RepoOwner::Organization(types::OrganizationOwner {
			id: organization_id.to_string(),
			name: organization_name.unwrap(),
		}),
		None => types::RepoOwner::User(types::UserOwner {
			email: user_email.unwrap(),
			id: user_id.unwrap().to_string(),
		}),
	};
	Ok(types::Repo {
		id: id.to_string(),
		title,
		models,
		owner,
	})
}

async fn get_models_for_repo(
	db: &deadpool_postgres::Transaction<'_>,
	repo_id: Id,
) -> Result<Vec<types::RepoModel>> {
	Ok(db
		.query(
			"
				select
					models.id,
					models.title,
					models.is_main
				from models
				join repos
					on models.repo_id = repos.id
				where
				repos.id = $1
			",
			&[&repo_id],
		)
		.await?
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let title: String = row.get(1);
			let is_main: bool = row.get(2);
			types::RepoModel {
				id: id.to_string(),
				title,
				is_main,
			}
		})
		.collect())
}
