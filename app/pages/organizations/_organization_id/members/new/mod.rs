use crate::{
	error::Error,
	user::{authorize_user, authorize_user_for_organization, User},
	Context,
};
use anyhow::{format_err, Result};
use chrono::Utc;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde_json::json;
use tangram_core::id::Id;

pub async fn get(
	_request: Request<Body>,
	context: &Context,
	_organization_id: &str,
) -> Result<Response<Body>> {
	let props = Props {};
	let html = context
		.pinwheel
		.render("/organizations/_organization_id/members/new", props)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(serde::Serialize)]
struct Props {}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Action {
	pub email: String,
	#[serde(rename = "isAdmin")]
	pub is_admin: Option<String>,
}

pub async fn post(
	mut request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	authorize_user_for_organization(&mut db, &user, organization_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let response = add_member(action, user, &mut db, context, organization_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn add_member(
	action: Action,
	user: User,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	organization_id: Id,
) -> Result<Response<Body>> {
	let Action { email, .. } = action;
	let inviter_email = user.email;
	if let Some(sendgrid_api_token) = context.sendgrid_api_token.clone() {
		tokio::spawn(send_invite_email(
			email.clone(),
			inviter_email.clone(),
			sendgrid_api_token,
		));
	}
	let user_id = Id::new();
	let now = Utc::now().timestamp();
	sqlx::query(
		"
		insert into users (
			id, created_at, email
		) values (
			?1, ?2, ?2
		)
		on conflict (email) do update set email = excluded.email
	",
	)
	.bind(&user_id.to_string())
	.bind(&now)
	.bind(&email)
	.execute(&mut *db)
	.await?;
	let is_admin = if let Some(is_admin) = action.is_admin {
		is_admin == "on"
	} else {
		false
	};
	sqlx::query(
		"
		insert into organizations_users
			(organization_id, user_id, is_admin)
		values
			(?1, ?2, ?3)
		on conflict (organization_id, user_id) do nothing
    ",
	)
	.bind(&organization_id.to_string())
	.bind(&user_id.to_string())
	.bind(&is_admin)
	.execute(&mut *db)
	.await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())?)
}

async fn send_invite_email(
	email: String,
	inviter_email: String,
	sendgrid_api_token: String,
) -> Result<()> {
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
		"subject": "Tangram Invite",
		"tracking_settings": {
			"click_tracking": {
				"enable": false
			}
		},
		"content": [
			{
				"type": "text/html",
				"value": format!("{} invited you to join their team on Tangram. <a href='https://app.tangramhq.com/login?email={}'>Accept Invitation</a>.", inviter_email, email),
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
		let text = response.text().await?;
		return Err(format_err!("Non-2xx response from sengrid: {:?}", text));
	}
	Ok(())
}
