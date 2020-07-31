use crate::{
	error::Error,
	helpers::organizations,
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, organization_id).await?;
	let html = context
		.pinwheel
		.render("/organizations/_organization_id/", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	card: Option<Card>,
	id: String,
	members: Vec<organizations::Member>,
	name: String,
	plan: organizations::Plan,
	user_id: String,
	repos: Vec<Repo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Repo {
	id: String,
	title: String,
	main_model_id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card {
	pub brand: String,
	pub country: String,
	pub exp_month: u8,
	pub exp_year: usize,
	pub last4: String,
	pub name: String,
}

async fn props(request: Request<Body>, context: &Context, organization_id: &str) -> Result<Props> {
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
	let organization = organizations::get_organization(organization_id, &db)
		.await?
		.ok_or(Error::NotFound)?;
	let card = get_card(
		&db,
		organization_id,
		context.stripe_secret_key.as_ref().unwrap(),
	)
	.await?;
	let repos = get_organization_repositories(&db, organization_id).await?;
	Ok(Props {
		id: organization_id.to_string(),
		name: organization.name,
		plan: organization.plan,
		members: organization.members,
		user_id: user.id.to_string(),
		card,
		repos,
	})
}

async fn get_organization_repositories(
	db: &deadpool_postgres::Transaction<'_>,
	organization_id: Id,
) -> Result<Vec<Repo>> {
	let rows = db
		.query(
			"
				select
					repos.id,
					repos.title,
					models.id
				from repos
				join models
					on models.repo_id = repos.id
					and models.is_main = 'true'
				where repos.organization_id = $1
      ",
			&[&organization_id],
		)
		.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: Id = row.get(0);
			let title: String = row.get(1);
			let main_model_id: Id = row.get(2);
			Repo {
				id: id.to_string(),
				title,
				main_model_id: main_model_id.to_string(),
			}
		})
		.collect())
}

#[derive(serde::Deserialize, Debug)]
struct BillingDetails {
	name: String,
}

#[derive(serde::Deserialize, Debug)]
struct StripePaymentMethodResponse {
	id: String,
	card: StripeCard,
	billing_details: BillingDetails,
}

#[derive(serde::Deserialize, Debug)]
struct StripeCard {
	brand: String,
	country: String,
	exp_month: u8,
	exp_year: usize,
	last4: String,
}

pub async fn get_card(
	db: &deadpool_postgres::Transaction<'_>,
	organization_id: Id,
	stripe_secret_key: &str,
) -> Result<Option<Card>> {
	let stripe_payment_method_id: Option<String> = db
		.query_one(
			"
        select
          organizations.stripe_payment_method_id
        from organizations
        where
          id = $1
      ",
			&[&organization_id],
		)
		.await?
		.try_get(0)
		.ok();
	match stripe_payment_method_id {
		Some(stripe_payment_method_id) => {
			let url = format!(
				"https://api.stripe.com/v1/payment_methods/{}",
				stripe_payment_method_id
			);
			let client = reqwest::Client::new();
			let response = client
				.get(&url)
				.basic_auth::<&str, &str>(stripe_secret_key, None)
				.send()
				.await?
				.json::<StripePaymentMethodResponse>()
				.await?;
			Ok(Some(Card {
				brand: response.card.brand,
				country: response.card.country,
				exp_month: response.card.exp_month,
				exp_year: response.card.exp_year,
				last4: response.card.last4,
				name: response.billing_details.name,
			}))
		}
		None => Ok(None),
	}
}
