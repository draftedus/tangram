use crate::common::{
	organizations::Plan,
	user::{authorize_user, authorize_user_for_organization, User},
};
use crate::{common::error::Error, Context};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde_json::json;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_organization")]
	DeleteOrganization,
	#[serde(rename = "change_plan")]
	ChangePlan(ChangePlanAction),
	#[serde(rename = "delete_member")]
	DeleteMember(DeleteMemberAction),
	#[serde(rename = "start_stripe_checkout")]
	StartStripeCheckout,
	#[serde(rename = "finish_stripe_checkout")]
	FinishStripeCheckout(FinishStripeCheckoutAction),
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangePlanAction {
	plan: Plan,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteMemberAction {
	member_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinishStripeCheckoutAction {
	stripe_checkout_session_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StartStripeCheckoutResponse {
	stripe_checkout_session_id: String,
}

pub async fn post(
	mut request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let user = user.unwrap();
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_organization(&mut db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
	let response = match action {
		Action::DeleteOrganization => delete_organization(&mut db, organization_id).await?,
		Action::DeleteMember(action) => delete_member(&mut db, organization_id, action).await?,
		Action::ChangePlan(action) => change_plan(&mut db, organization_id, action).await?,
		Action::StartStripeCheckout => {
			start_stripe_checkout(&mut db, organization_id, user, context).await?
		}
		Action::FinishStripeCheckout(action) => {
			finish_stripe_checkout(&mut db, organization_id, action, context).await?
		}
	};
	db.commit().await?;
	Ok(response)
}

async fn delete_organization(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<Response<Body>> {
	sqlx::query(
		"
		delete from organizations
		where
			id = ?1
	",
	)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/user")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn delete_member(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	action: DeleteMemberAction,
) -> Result<Response<Body>> {
	let DeleteMemberAction { member_id } = action;
	let member_id: Id = member_id.parse().map_err(|_| Error::NotFound)?;
	sqlx::query(
		"
		delete from organizations_users
		where
			organization_id = ?1
			and user_id = ?2
	",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/user")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn change_plan(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	action: ChangePlanAction,
) -> Result<Response<Body>> {
	let ChangePlanAction { plan } = action;
	let plan = match plan {
		Plan::Trial => "trial",
		Plan::Startup => "startup",
		Plan::Team => "team",
		Plan::Enterprise => "Enterprise",
	};
	sqlx::query(
		"
		update organizations
			set plan = ?1
		where organizations.id = ?2
	",
	)
	.bind(&plan)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
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

async fn start_stripe_checkout(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	user: User,
	context: &Context,
) -> Result<Response<Body>> {
	// Retrieve the existing stripe customer id for the organization.
	let existing_stripe_customer_id: Option<String> = sqlx::query(
		"
			select
				organizations.stripe_customer_id
			from organizations
			where
				id = ?1
			and organizations.stripe_customer_id is not null
		",
	)
	.bind(&organization_id.to_string())
	.fetch_optional(&mut *db)
	.await?
	.and_then(|r| r.get(0));
	// Retrieve or create the stripe customer.
	let stripe_customer_id = match existing_stripe_customer_id {
		Some(stripe_customer_id) => stripe_customer_id,
		None => {
			let client = reqwest::Client::new();
			// Create a stripe customer.
			let json = json!({
				"email": &user.email,
			});
			let response = client
				.post("https://api.stripe.com/v1/customers")
				.basic_auth::<&str, &str>(context.options.stripe_secret_key.as_ref().unwrap(), None)
				.form(&json)
				.send()
				.await?
				.json::<serde_json::Value>()
				.await?;
			let stripe_customer_id = response.get("id").unwrap().as_str().unwrap().to_owned();
			// Save the stripe customer id with the tangram user.
			sqlx::query(
				"
					update organizations
						set stripe_customer_id = ?1
					where
						id = ?2
				",
			)
			.bind(&stripe_customer_id)
			.bind(&organization_id.to_string())
			.execute(&mut *db)
			.await?;
			stripe_customer_id
		}
	};
	// Create the checkout session.
	let base_url = context.options.url.as_ref().unwrap();
	let json = json!({
		"payment_method_types[]": "card",
		"mode": "setup",
		"customer": stripe_customer_id,
		"success_url": base_url.join(&format!("organizations/{}/?session_id={{CHECKOUT_SESSION_ID}}", organization_id)).ok().unwrap().to_string(),
		"cancel_url": base_url.join(&format!("organizations/{}/", organization_id)).ok().unwrap().to_string(),
	});
	let client = reqwest::Client::new();
	let response = client
		.post("https://api.stripe.com/v1/checkout/sessions")
		.basic_auth::<&str, &str>(context.options.stripe_secret_key.as_ref().unwrap(), None)
		.form(&json)
		.send()
		.await?
		.json::<serde_json::Value>()
		.await?;
	let session_id = response.get("id").unwrap().as_str().unwrap().to_owned();
	let response = StartStripeCheckoutResponse {
		stripe_checkout_session_id: session_id,
	};
	let response = serde_json::to_vec(&response)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))
		.unwrap();
	Ok(response)
}

async fn finish_stripe_checkout(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	action: FinishStripeCheckoutAction,
	context: &Context,
) -> Result<Response<Body>> {
	#[derive(serde::Deserialize)]
	struct SessionResponse {
		setup_intent: PaymentMethod,
	}
	#[derive(serde::Deserialize)]
	struct PaymentMethod {
		payment_method: String,
	}
	let json = json!({
		"expand[]": "setup_intent"
	});
	let url = format!(
		"https://api.stripe.com/v1/checkout/sessions/{}",
		action.stripe_checkout_session_id
	);
	let client = reqwest::Client::new();
	let response = client
		.get(&url)
		.basic_auth::<&str, &str>(context.options.stripe_secret_key.as_ref().unwrap(), None)
		.form(&json)
		.send()
		.await?
		.json::<SessionResponse>()
		.await?;
	let stripe_payment_method_id = response.setup_intent.payment_method;
	sqlx::query(
		"
			update organizations set
				stripe_payment_method_id = ?1
			where
				id = ?2
		",
	)
	.bind(&stripe_payment_method_id)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
