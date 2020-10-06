use crate::{common::error::Error, Context};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use rand::Rng;
use serde_json::json;
use sqlx::prelude::*;
use std::{collections::BTreeMap, sync::Arc};
use tangram_id::Id;

pub async fn get(
	_request: Request<Body>,
	context: Arc<Context>,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let email = search_params.as_ref().and_then(|s| s.get("email").cloned());
	let props = LoginProps {
		code: email.is_some(),
		error: None,
		email,
	};
	let html = context.pinwheel.render_with("/login", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(serde::Serialize)]
struct LoginProps {
	code: bool,
	email: Option<String>,
	error: Option<String>,
}

#[derive(serde::Deserialize)]
struct Action {
	email: String,
	code: Option<String>,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	// read the post data
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let Action { email, code } =
		serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	// upsert the user
	let user_id = Id::new();
	let now = Utc::now().timestamp();
	sqlx::query(
		"
			insert into users (
				id, created_at, email
			) values (
				?1, ?2, ?3
			)
			on conflict (email) do update set email = excluded.email
		",
	)
	.bind(&user_id.to_string())
	.bind(&now)
	.bind(&email)
	.execute(&mut *db)
	.await?;
	// retrieve the user's id
	let user_id: String = sqlx::query(
		"
			select
				id
			from users
			where
				email = ?1
		",
	)
	.bind(&email)
	.fetch_one(&mut *db)
	.await?
	.get(0);
	let user_id: Id = user_id.parse()?;
	if context.options.auth_enabled {
		if let Some(code) = code {
			// verify the code
			let ten_minutes_in_seconds: i32 = 10 * 60;
			let now = Utc::now().timestamp();
			let row = sqlx::query(
				"
					select
						codes.id as code_id
					from users
					join codes
					on codes.user_id = users.id
					where
						codes.deleted_at is null and
						?1 - codes.created_at < ?2 and
						users.email = ?3 and
						codes.code = ?4
				",
			)
			.bind(&now)
			.bind(&ten_minutes_in_seconds)
			.bind(&email)
			.bind(&code)
			.fetch_optional(&mut db)
			.await?;
			println!("{:?}", code);
			println!("{:?}", email);
			let row = if let Some(row) = row {
				row
			} else {
				let props = LoginProps {
					code: true,
					error: Some("invalid code".into()),
					email: Some(email),
				};
				let html = context.pinwheel.render_with("/login", props)?;
				let response = Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.body(Body::from(html))?;
				return Ok(response);
			};
			let code_id: String = row.get(0);
			let code_id: Id = code_id.parse()?;
			let now = Utc::now().timestamp();
			// delete the code
			sqlx::query(
				"
					update codes
					set
						deleted_at = ?1
					where
						id = ?2
				",
			)
			.bind(&now)
			.bind(&code_id.to_string())
			.execute(&mut db)
			.await?;
		} else {
			// generate a code and redirect back to the login form
			let code: u64 = rand::thread_rng().gen_range(0, 1_000_000);
			let code = format!("{:06}", code);
			let now = Utc::now().timestamp();
			let code_id = Id::new();
			sqlx::query(
				"
					insert into codes (
						id, created_at, user_id, code
					) values (
						?1, ?2, ?3, ?4
					)
				",
			)
			.bind(&code_id.to_string())
			.bind(&now)
			.bind(&user_id.to_string())
			.bind(&code)
			.execute(&mut *db)
			.await?;
			if let Some(sendgrid_api_token) = context.options.sendgrid_api_token.clone() {
				send_code_email(email.to_owned(), code, sendgrid_api_token).await?;
			}
			db.commit().await?;
			let response = Response::builder()
				.status(StatusCode::SEE_OTHER)
				.header(header::LOCATION, format!("/login?email={}", email))
				.body(Body::empty())?;
			return Ok(response);
		}
	}

	let token = create_token(&mut db, user_id).await?;

	db.commit().await?;

	let set_cookie = set_cookie_header_value(token, context.options.cookie_domain.as_deref());
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.header(header::SET_COOKIE, set_cookie)
		.body(Body::empty())?;

	Ok(response)
}

async fn create_token(db: &mut sqlx::Transaction<'_, sqlx::Any>, user_id: Id) -> Result<Id> {
	let id = Id::new();
	let token = Id::new();
	let now = Utc::now().timestamp();
	sqlx::query(
		"
			insert into tokens (
				id, created_at, token, user_id
			) values (
				?1, ?2, ?3, ?4
			)
		",
	)
	.bind(&id.to_string())
	.bind(&now)
	.bind(&token.to_string())
	.bind(&user_id.to_string())
	.execute(db)
	.await?;
	Ok(token)
}

fn set_cookie_header_value(token: Id, domain: Option<&str>) -> String {
	let domain = domain.map(|domain| format!(";domain={}", domain));
	let path = Some(";path=/");
	let max_age = Some(";max-age=31536000");
	let same_site = if domain.is_some() {
		Some(";samesite=none")
	} else {
		None
	};
	let secure = if domain.is_some() {
		Some(";secure")
	} else {
		None
	};
	format!(
		"tangram-auth={}{}{}{}{}{}",
		token,
		domain.as_deref().unwrap_or(""),
		path.unwrap_or(""),
		max_age.unwrap_or(""),
		same_site.unwrap_or(""),
		secure.unwrap_or("")
	)
}

async fn send_code_email(email: String, code: String, sendgrid_api_token: String) -> Result<()> {
	let json = json!({
		"personalizations": [
			{
				"to": [
					{
						"email": email,
					}
				]
			}
		],
		"from": {
			"email": "noreply@tangramhq.com",
			"name": "Tangram"
		},
		"subject": "Your Tangram Login Code",
		"tracking_settings": {
			"click_tracking": {
				"enable": false
			}
		},
		"content": [
			{
				"type": "text/plain",
				"value": format!("Your Tangram login code is: {}", code),
			}
		]
	});
	let client = reqwest::Client::new();
	let response = client
		.post("https://api.sendgrid.com/v3/mail/send")
		.header(
			reqwest::header::AUTHORIZATION,
			format!("Bearer {}", sendgrid_api_token),
		)
		.json(&json)
		.send()
		.await?;
	if !response.status().is_success() {
		panic!(
			"Non-2xx response from sengrid: {:?}",
			response.text().await?
		);
	}
	Ok(())
}
