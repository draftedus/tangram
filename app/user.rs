use crate::app::cookies;
use anyhow::Result;
use hyper::{header, Body, Request};
use tangram::id::Id;
use tokio_postgres as postgres;

#[derive(Debug)]
pub struct User {
	pub id: Id,
	pub email: String,
	pub token: String,
}

pub enum AuthorizeUserError {
	CookieAndAuthorizationHeadersAbsent,
	AuthorizationNotString,
	AuthorizationInvalid,
	CookieNotString,
	CookieParseFailed,
	CookieAuthAbsent,
	TokenUnknown,
}

pub async fn authorize_user(
	request: &Request<Body>,
	db: &postgres::Transaction<'_>,
) -> Result<Result<User, AuthorizeUserError>> {
	let token = if let Some(authorization) = request.headers().get(header::AUTHORIZATION) {
		let authorization = match authorization.to_str() {
			Ok(authorization) => authorization,
			Err(_) => return Ok(Err(AuthorizeUserError::AuthorizationNotString)),
		};
		let mut components = authorization.split(' ');
		match (components.next(), components.next()) {
			(Some("Bearer"), Some(token)) => token.to_string(),
			_ => return Ok(Err(AuthorizeUserError::AuthorizationInvalid)),
		}
	} else if let Some(cookies) = request.headers().get(header::COOKIE) {
		let cookies = match cookies.to_str() {
			Ok(cookies) => cookies,
			Err(_) => return Ok(Err(AuthorizeUserError::CookieNotString)),
		};
		let cookies = match cookies::parse(cookies) {
			Ok(cookies) => cookies,
			Err(_) => return Ok(Err(AuthorizeUserError::CookieParseFailed)),
		};
		match cookies.get("tangram-auth") {
			Some(&auth_cookie) => auth_cookie.to_owned(),
			None => return Ok(Err(AuthorizeUserError::CookieAuthAbsent)),
		}
	} else {
		return Ok(Err(AuthorizeUserError::CookieAndAuthorizationHeadersAbsent));
	};
	let user_rows = db
		.query(
			"
				select
					users.id, users.email
				from tokens
				join users
					on users.id = tokens.user_id
				where
					tokens.token = $1 and
					tokens.deleted_at is null
			",
			&[&token],
		)
		.await?;
	let user = match user_rows.iter().next() {
		Some(row) => User {
			id: row.get(0),
			email: row.get(1),
			token,
		},
		None => return Ok(Err(AuthorizeUserError::TokenUnknown)),
	};
	Ok(Ok(user))
}

pub async fn authorize_user_for_organization(
	db: &postgres::Transaction<'_>,
	user: &User,
	organization_id: Id,
) -> Result<bool> {
	Ok(db
		.query_one(
			"
				select
					count(*) > 0
				from organizations_users
				where organizations_users.user_id = $1
					and organizations_users.organization_id = $2
			",
			&[&user.id, &organization_id],
		)
		.await?
		.get(0))
}

pub async fn authorize_user_for_model(
	db: &postgres::Transaction<'_>,
	user: &User,
	model_id: Id,
) -> Result<bool> {
	Ok(db
		.query_one(
			"
				select
					count(*) > 0
				from models
				join repos
					on repos.id = models.repo_id
				left join users
					on users.id = repos.user_id
				left join organizations_users on
					organizations_users.organization_id = repos.organization_id
				and
					organizations_users.user_id = $1
				where
					models.id = $2
			",
			&[&user.id, &model_id],
		)
		.await?
		.get(0))
}

pub async fn authorize_user_for_repo(
	db: &postgres::Transaction<'_>,
	user: &User,
	repo_id: Id,
) -> Result<bool> {
	Ok(db
		.query_one(
			"
				select
					count(*) > 0
				from repos
				left join users
					on users.id = repos.user_id
				left join organizations_users
				 on organizations_users.organization_id = repos.organization_id
				and
					organizations_users.user_id = $1
				where
					repos.id = $2
			",
			&[&user.id, &repo_id],
		)
		.await?
		.get(0))
}
