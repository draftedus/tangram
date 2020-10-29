use crate::{
	common::organizations::{Member, Plan},
	layouts::app_layout::AppLayoutInfo,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub card: Option<Card>,
	pub id: String,
	pub members: Vec<Member>,
	pub name: String,
	pub plan: Plan,
	pub repos: Vec<Repo>,
	pub stripe_publishable_key: String,
	pub user_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
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

#[derive(serde::Deserialize, Debug)]
pub struct StripePaymentMethodResponse {
	pub id: String,
	pub card: StripeCard,
	pub billing_details: BillingDetails,
}

#[derive(serde::Deserialize, Debug)]
pub struct BillingDetails {
	pub name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct StripeCard {
	pub brand: String,
	pub country: String,
	pub exp_month: u8,
	pub exp_year: usize,
	pub last4: String,
}
