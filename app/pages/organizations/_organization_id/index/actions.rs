use crate::app::{
	error::Error,
	helpers::organizations,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde_json::json;
use std::sync::Arc;
use tangram::id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_organization")]
	DeleteOrganization,
	#[serde(rename = "change_plan")]
	ChangePlan(ChangePlanAction),
	#[serde(rename = "delete_member")]
	DeleteMember(DeleteMemberAction),
}

#[derive(serde::Deserialize)]
struct ChangePlanAction {
	plan: organizations::Plan,
}

#[derive(serde::Deserialize)]
struct DeleteMemberAction {
	#[serde(rename = "memberId")]
	member_id: String,
}

pub async fn actions(
	mut request: Request<Body>,
	context: Arc<Context>,
	organization_id: &str,
) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_organization(&db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
	match action {
		Action::DeleteOrganization => delete_organization(db, organization_id).await,
		Action::DeleteMember(action) => delete_member(action, db, organization_id).await,
		Action::ChangePlan(action) => change_plan(action, db, organization_id).await,
	}
}

async fn delete_organization(
	db: deadpool_postgres::Transaction<'_>,
	organization_id: Id,
) -> Result<Response<Body>> {
	db.query(
		"
			delete from organizations
			where
				id = $1
		",
		&[&organization_id],
	)
	.await?;
	db.commit().await?;

	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/user/")
		.body(Body::empty())?)
}

async fn delete_member(
	action: DeleteMemberAction,
	db: deadpool_postgres::Transaction<'_>,
	organization_id: Id,
) -> Result<Response<Body>> {
	let DeleteMemberAction { member_id } = action;
	let member_id: Id = member_id.parse().map_err(|_| Error::NotFound)?;
	db.execute(
		"
			delete from organizations_users
			where
				organization_id = $1
				and user_id = $2
		",
		&[&organization_id, &member_id],
	)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())?)
}

async fn change_plan(
	action: ChangePlanAction,
	db: deadpool_postgres::Transaction<'_>,
	organization_id: Id,
) -> Result<Response<Body>> {
	let ChangePlanAction { plan } = action;

	db.execute(
		"
			update organizations
				set plan = $1
			where organizations.id = $2
		",
		&[&plan, &organization_id],
	)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())?)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartStripeCheckoutResponse {
	pub stripe_checkout_session_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishStripeCheckoutRequest {
	pub stripe_checkout_session_id: String,
}

pub async fn start_stripe_checkout(
	request: Request<Body>,
	context: Arc<Context>,
) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	// TODO
	let organization_id = Id::new();
	if !authorize_user_for_organization(&db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
	// retrieve the existing stripe customer id for the organization
	let existing_stripe_customer_id: Option<String> = db
		.query_one(
			"
        select
          organizations.stripe_customer_id
        from organizations
        where
          id = $1
      ",
			&[&organization_id],
		)
		.await?
		.try_get(0)
		.ok();
	// retrieve or create the stripe customer
	let stripe_customer_id = match existing_stripe_customer_id {
		Some(s) => s,
		None => {
			let client = reqwest::Client::new();
			// create a stripe customer
			let json = json!({
				"email": &user.email,
			});
			let response = client
				.post("https://api.stripe.com/v1/customers")
				.basic_auth::<&str, &str>(context.stripe_secret_key.as_ref().unwrap(), None)
				.form(&json)
				.send()
				.await?
				.json::<serde_json::Value>()
				.await?;
			let stripe_customer_id = response.get("id").unwrap().as_str().unwrap().to_owned();
			// save the stripe customer id with the tangram user
			db.execute(
				"
          update organizations
            set stripe_customer_id = $1
          where
            id = $2
        ",
				&[&stripe_customer_id, &organization_id],
			)
			.await?;
			stripe_customer_id
		}
	};
	// create the checkout session
	let json = json!({
		"payment_method_types[]": "card",
		"mode": "setup",
		"customer": stripe_customer_id,
		"success_url": format!("{}/organizations/{}/?session_id={{CHECKOUT_SESSION_ID}}", context.app_url.as_ref().unwrap(), organization_id),
		"cancel_url": format!("{}/organizations/{}/", context.app_url.as_ref().unwrap(), organization_id)
	});
	let client = reqwest::Client::new();
	let response = client
		.post("https://api.stripe.com/v1/checkout/sessions")
		.basic_auth::<&str, &str>(context.stripe_secret_key.as_ref().unwrap(), None)
		.form(&json)
		.send()
		.await?
		.json::<serde_json::Value>()
		.await?;
	let session_id = response.get("id").unwrap().as_str().unwrap().to_owned();
	db.commit().await?;
	let response = StartStripeCheckoutResponse {
		stripe_checkout_session_id: session_id,
	};
	let response = serde_json::to_vec(&response)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}

pub async fn finish_stripe_checkout(
	mut request: Request<Body>,
	context: Arc<Context>,
	organization_id: &str,
) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let data: FinishStripeCheckoutRequest =
		serde_json::from_slice(&data).map_err(|_| Error::BadRequest)?;
	let session_id = data.stripe_checkout_session_id;
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_organization(&db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
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
	let url = format!("https://api.stripe.com/v1/checkout/sessions/{}", session_id);
	let client = reqwest::Client::new();
	let response = client
		.get(&url)
		.basic_auth::<&str, &str>(context.stripe_secret_key.as_ref().unwrap(), None)
		.form(&json)
		.send()
		.await?
		.json::<SessionResponse>()
		.await?;
	let stripe_payment_method_id = response.setup_intent.payment_method;
	db.execute(
		"
      update organizations
        set stripe_payment_method_id = $1
      where
        id = $2
    ",
		&[&stripe_payment_method_id, &organization_id],
	)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::empty())?)
}
