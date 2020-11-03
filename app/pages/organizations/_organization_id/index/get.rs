use super::props::{Card, Props, Repo, StripePaymentMethodResponse};
use crate::{
	common::{
		error::Error,
		organizations::get_organization,
		user::{authorize_normal_user, authorize_normal_user_for_organization},
	},
	layouts::app_layout::get_app_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
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
			where repos.organization_id = $1
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
	let props = Props {
		app_layout_info,
		card,
		id: organization_id.to_string(),
		members: organization.members,
		name: organization.name,
		plan: organization.plan,
		repos,
		stripe_publishable_key,
		user_id: user.id.to_string(),
	};
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
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
				id = $1
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
