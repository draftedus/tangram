use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	user::NormalUser,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_deps::sqlx::*;
use tangram_deps::{
	http, hyper, pinwheel::Pinwheel, reqwest, serde_json::json, serde_urlencoded, sqlx,
};
use tangram_util::{err, error::Result, id::Id};

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Action {
	email: String,
	is_admin: Option<String>,
}

pub async fn post(
	_pinwheel: &Pinwheel,
	context: &Context,
	mut request: http::Request<hyper::Body>,
	organization_id: &str,
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(&request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	};
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
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
) -> Result<http::Response<hyper::Body>> {
	// Create the new user.
	let user_id = Id::new();
	sqlx::query(
		"
			insert into users (
				id, email
			) values (
				$1, $2
			)
			on conflict (email) do update set email = excluded.email
		",
	)
	.bind(&user_id.to_string())
	.bind(&action.email)
	.execute(&mut *db)
	.await?;
	let row = sqlx::query(
		"
			select id
				from users
			where email = $1
		",
	)
	.bind(&action.email)
	.fetch_one(&mut *db)
	.await?;
	let user_id: String = row.get(0);
	let user_id: Id = user_id.parse().unwrap();
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
				($1, $2, $3)
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
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(hyper::Body::empty())
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
			http::header::AUTHORIZATION,
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
