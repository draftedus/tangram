use crate::{common::organizations::Organization, layouts::app_layout::AppLayoutInfo};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	#[serde(rename = "auth")]
	Auth(AuthProps),
	#[serde(rename = "no_auth")]
	NoAuth(NoAuthProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProps {
	pub email: String,
	pub organizations: Vec<Organization>,
	pub repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoAuthProps {
	pub repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
}
