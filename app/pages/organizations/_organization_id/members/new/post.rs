use crate::common::{
	error::Error,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
};
use crate::{common::user::NormalUser, Context};
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde_json::json;
use tangram_util::id::Id;
use tangram_util::{err, error::Result};

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Action {
	email: String,
	is_admin: Option<String>,
}

pub async fn post(
	context: &Context,
	mut request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	authorize_normal_user_for_organization(&mut db, &user, organization_id)
		.await
		.map_err(|_| Error::NotFound)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let response = add_member(action, user, &mut db, context, organization_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn add_member(
	action: Action,
	user: NormalUser,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	organization_id: Id,
) -> Result<Response<Body>> {
	// Create the new user.
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
	.bind(&action.email)
	.execute(&mut *db)
	.await?;
	// Add the user to the organization.
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
	// Send the new user an invitation email.
	if let Some(sendgrid_api_token) = context.options.sendgrid_api_token.clone() {
		send_invitation_email(action.email.clone(), user.email.clone(), sendgrid_api_token).await?;
	}
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn send_invitation_email(
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
		return Err(err!("Non-2xx response from sengrid: {:?}", text));
	}
	Ok(())
}
