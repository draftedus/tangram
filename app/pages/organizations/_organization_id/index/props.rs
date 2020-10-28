use crate::{
	common::{
		error::Error,
		organizations::{get_organization, Member, Plan},
		user::{authorize_normal_user, authorize_normal_user_for_organization},
	},
	layouts::app_layout::{get_app_layout_info, AppLayoutInfo},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	app_layout_info: AppLayoutInfo,
	card: Option<Card>,
	id: String,
	members: Vec<Member>,
	name: String,
	plan: Plan,
	repos: Vec<Repo>,
	stripe_publishable_key: String,
	user_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	id: String,
	title: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card {
	brand: String,
	country: String,
	exp_month: u8,
	exp_year: usize,
	last4: String,
	name: String,
}

#[derive(serde::Deserialize, Debug)]
struct StripePaymentMethodResponse {
	id: String,
	card: StripeCard,
	billing_details: BillingDetails,
}

#[derive(serde::Deserialize, Debug)]
struct BillingDetails {
	name: String,
}

#[derive(serde::Deserialize, Debug)]
struct StripeCard {
	brand: String,
	country: String,
	exp_month: u8,
	exp_year: usize,
	last4: String,
}

pub async fn props(
	context: &Context,
	request: Request<Body>,
	organization_id: &str,
) -> Result<Props> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let app_layout_info = get_app_layout_info(context).await?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
	}
	let organization = get_organization(organization_id, &mut db)
		.await?
		.ok_or(Error::NotFound)?;
	let card = get_card(
		&mut db,
		organization_id,
		context.options.stripe_secret_key.as_ref().unwrap(),
	)
	.await?;
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			join models
				on models.repo_id = repos.id
			where repos.organization_id = ?1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let repos = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect();
	let stripe_publishable_key = context
		.options
		.stripe_publishable_key
		.as_ref()
		.unwrap()
		.to_owned();
	Ok(Props {
		app_layout_info,
		card,
		id: organization_id.to_string(),
		members: organization.members,
		name: organization.name,
		plan: organization.plan,
		repos,
		stripe_publishable_key,
		user_id: user.id.to_string(),
	})
}

async fn get_card(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	stripe_secret_key: &str,
) -> Result<Option<Card>> {
	let row = sqlx::query(
		"
			select
				organizations.stripe_payment_method_id
			from organizations
			where
				id = ?1
			and organizations.stripe_payment_method_id is not null
		",
	)
	.bind(&organization_id.to_string())
	.fetch_optional(&mut *db)
	.await?;
	let stripe_payment_method_id: Option<String> = row.map(|r| r.get(0));
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
