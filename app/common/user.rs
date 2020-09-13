use crate::common::cookies::parse_cookies;
use anyhow::Result;
use hyper::{header, Body, Request};
use sqlx::prelude::*;
use tangram_core::util::id::Id;

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
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	auth_enabled: bool,
) -> Result<Result<Option<User>, AuthorizeUserError>> {
	// when auth is disabled, everyone is authorized as the root user
	if !auth_enabled {
		return Ok(Ok(None));
	}
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
		let cookies = match parse_cookies(cookies) {
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
	let row = sqlx::query(
		"
			select
				users.id, users.email
			from tokens
			join users
				on users.id = tokens.user_id
			where
				tokens.token = ?1 and
				tokens.deleted_at is null
		",
	)
	.bind(&token)
	.fetch_optional(db)
	.await?;
	let row = if let Some(row) = row {
		row
	} else {
		return Ok(Err(AuthorizeUserError::TokenUnknown));
	};
	let id: String = row.get(0);
	let id: Id = id.parse().unwrap();
	let email = row.get(1);
	let user = User { id, email, token };
	Ok(Ok(Some(user)))
}

pub async fn authorize_user_for_organization(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	organization_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
		"
			select
				count(*) > 0
			from organizations_users
			where organizations_users.user_id = ?1
				and organizations_users.organization_id = ?2
		",
	)
	.bind(&user.id.to_string())
	.bind(&organization_id.to_string())
	.fetch_one(&mut *db)
	.await?
	.get(0))
}

pub async fn authorize_user_for_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	model_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
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
				organizations_users.user_id = ?1
			where
				models.id = ?2
		",
	)
	.bind(&user.id.to_string())
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?
	.get(0))
}

pub async fn authorize_user_for_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	repo_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
		"
			select
				count(*) > 0
			from repos
			left join users
				on users.id = repos.user_id
			left join organizations_users
				on organizations_users.organization_id = repos.organization_id
			and
				organizations_users.user_id = ?1
			where
				repos.id = ?2
		",
	)
	.bind(&user.id.to_string())
	.bind(&repo_id.to_string())
	.fetch_one(&mut *db)
	.await?
	.get(0))
}
