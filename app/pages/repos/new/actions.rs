use crate::app::types;
use crate::app::{
	error::Error,
	user::{authorize_user, authorize_user_for_organization, User},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram::id::Id;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
	pub data: String,
	pub organization_id: Option<String>,
	pub title: String,
}

pub async fn actions(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	create_repo(action, db, user).await
}

async fn create_repo(
	action: Action,
	db: deadpool_postgres::Transaction<'_>,
	user: User,
) -> Result<Response<Body>> {
	let Action {
		title,
		data,
		organization_id,
	} = action;

	let organization_id: Option<Id> = if let Some(organization_id) = organization_id {
		match organization_id.parse() {
			Ok(organization_id) => {
				if !authorize_user_for_organization(&db, &user, organization_id).await? {
					return Err(Error::Unauthorized.into());
				}
				Some(organization_id)
			}
			_ => return Err(Error::BadRequest.into()),
		}
	} else {
		None
	};

	let model_data: Vec<u8> = base64::decode(&data).map_err(|_| Error::BadRequest)?;
	let model = tangram::types::Model::from_slice(&model_data).map_err(|_| Error::BadRequest)?;

	let created_at: DateTime<Utc> = Utc::now();

	let user_id = if organization_id.is_none() {
		Some(user.id)
	} else {
		None
	};

	// create the repo
	let repo_id: Id = db
		.query_one(
			"
				insert into repos (
					id, created_at, title, organization_id, user_id
				) values (
					$1, $2, $3, $4, $5
				)
				returning id
			",
			&[
				&Id::new().to_string(),
				&created_at,
				&title,
				&organization_id,
				&user_id,
			],
		)
		.await?
		.get(0);

	// insert the model
	db.execute(
		"
			insert into models
				(id, repo_id, title, created_at, data, is_main)
			values
				($1, $2, $3, $4, $5, $6)
		",
		&[
			&model.id(),
			&repo_id,
			&title,
			&created_at,
			&model_data,
			&true,
		],
	)
	.await?;
	db.commit().await?;

	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, format!("/repos/{}", repo_id))
		.body(Body::empty())?)
}

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
