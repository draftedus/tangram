use crate::app::{error::Error, Context};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use rand::Rng;
use serde_json::json;
use tangram::id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
pub enum Action {
	#[serde(rename = "email")]
	Email(EmailAction),
	#[serde(rename = "code")]
	Code(CodeAction),
}

#[derive(serde::Deserialize)]
pub struct EmailAction {
	pub email: String,
}

#[derive(serde::Deserialize)]
pub struct CodeAction {
	pub email: String,
	pub code: String,
}

pub async fn actions(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let request_body: Action =
		serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	match request_body {
		Action::Email(request_body) => email(request_body, db, context).await,
		Action::Code(request_body) => code(request_body, db, context).await,
	}
}

pub async fn email(
	request_body: EmailAction,
	db: deadpool_postgres::Transaction<'_>,
	context: &Context,
) -> Result<Response<Body>> {
	let EmailAction { email } = request_body;
	let user_id: Id = db
		.query_one(
			"
				insert into users (
					id, created_at, email
				) values (
					$1, now(), $2
				)
				on conflict (email) do update set email = excluded.email
				returning id
			",
			&[&Id::new().to_string(), &email],
		)
		.await?
		.get(0);
	if context.auth_enabled {
		let code: u64 = rand::thread_rng().gen_range(0, 1_000_000);
		let code = format!("{:06}", code);
		let code_id = Id::new();
		db.execute(
			"
				insert into codes (
					id, created_at, user_id, code
				) values (
					$1, now(), $2, $3
				)
			",
			&[&code_id.to_string(), &user_id, &code],
		)
		.await?;
		if let Some(sendgrid_api_token) = context.sendgrid_api_token.clone() {
			tokio::spawn(send_code_email(email.to_owned(), code, sendgrid_api_token));
		}
		db.commit().await?;
		let response = Response::builder()
			.status(StatusCode::SEE_OTHER)
			.header(header::LOCATION, format!("/login?email={}", email))
			.body(Body::empty())?;
		Ok(response)
	} else {
		// create the token
		let id = Id::new();
		let token = Id::new();
		db.execute(
			"
			insert into tokens (
				id, created_at, token, user_id
			) values (
				$1, now(), $2, $3
			)
		",
			&[&id, &token, &user_id],
		)
		.await?;
		db.commit().await?;
		let set_cookie = set_cookie_header_value(token, context.cookie_domain.as_deref());
		let response = Response::builder()
			.status(StatusCode::SEE_OTHER)
			.header(header::LOCATION, "/")
			.header(header::SET_COOKIE, set_cookie)
			.body(Body::empty())?;
		Ok(response)
	}
}

pub async fn code(
	request_body: CodeAction,
	db: deadpool_postgres::Transaction<'_>,
	context: &Context,
) -> Result<Response<Body>> {
	let CodeAction { email, code } = request_body;
	let user_id = if context.auth_enabled {
		let rows = db
			.query(
				"
					select
						users.id as user_id,
						codes.id as code_id
					from users
					join codes
					on codes.user_id = users.id
					where
						codes.deleted_at is null and
						age(now(), codes.created_at) < interval '10 minutes' and
						users.email = $1 and
						codes.code = $2
				",
				&[&email, &code],
			)
			.await?;
		let row = rows.iter().next().ok_or(Error::Unauthorized)?;
		let user_id: String = row.get(0);
		let user_id: Id = user_id.parse()?;
		let code_id: Id = row.get(1);
		// delete the code
		db.execute(
			"
				update codes
				set
					deleted_at = now()
				where
					id = $1
			",
			&[&code_id.to_string()],
		)
		.await?;
		user_id
	} else {
		let rows = db
			.query(
				"
					select
						id
					from users
					where
						users.email = $1
				",
				&[&email],
			)
			.await?;
		let row = rows.iter().next().ok_or(Error::Unauthorized)?;
		let user_id: Id = row.get(0);
		user_id
	};

	// create the token
	let id = Id::new();
	let token = Id::new();
	db.execute(
		"
			insert into tokens (
				id, created_at, token, user_id
			) values (
				$1, now(), $2, $3
			)
		",
		&[&id, &token, &user_id],
	)
	.await?;
	db.commit().await?;
	let set_cookie = set_cookie_header_value(token, context.cookie_domain.as_deref());

	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.header(header::SET_COOKIE, set_cookie)
		.body(Body::empty())?)
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
