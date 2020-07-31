use crate::{
	error::Error,
	types,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use tangram_core::id::Id;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
	pub data: String,
	pub organization_id: Option<String>,
	pub title: String,
}

pub async fn actions(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok());
	let boundary = boundary.ok_or_else(|| Error::BadRequest)?;
	let mut title: Option<String> = None;
	let mut organization_id: Option<String> = None;
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = field.name().ok_or_else(|| Error::BadRequest)?.to_owned();
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"title" => title = Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?),
			"organization_id" => {
				organization_id =
					Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?)
			}
			"file" => file = Some(field_data),
			_ => return Err(Error::BadRequest.into()),
		};
	}
	let title = title.ok_or_else(|| Error::BadRequest)?;
	let organization_id: Id = organization_id
		.ok_or(Error::BadRequest)?
		.parse()
		.map_err(|_| Error::BadRequest)?;
	let file = file.ok_or_else(|| Error::BadRequest)?;
	if !authorize_user_for_organization(&db, &user, organization_id).await? {
		return Err(Error::Unauthorized.into());
	}
	let model = tangram_core::types::Model::from_slice(&file).map_err(|_| Error::BadRequest)?;
	let created_at: DateTime<Utc> = Utc::now();
	let repo_id: Id = db
		.query_one(
			"
				insert into repos (
					id, created_at, title, organization_id
				) values (
					$1, $2, $3, $4
				)
				returning id
			",
			&[
				&Id::new().to_string(),
				&created_at,
				&title,
				&organization_id,
			],
		)
		.await?
		.get(0);
	db.execute(
		"
			insert into models
				(id, repo_id, title, created_at, data, is_main)
			values
				($1, $2, $3, $4, $5, $6)
		",
		&[&model.id(), &repo_id, &title, &created_at, &file, &true],
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
