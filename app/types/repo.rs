#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
	pub id: String,
	pub name: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
	pub models: Vec<RepoModel>,
	pub owner: RepoOwner,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum RepoOwner {
	Organization(OrganizationOwner),
	User(UserOwner),
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserOwner {
	pub email: String,
	pub id: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationOwner {
	pub id: String,
	pub name: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepoModel {
	pub id: String,
	pub title: String,
	pub is_main: bool,
}
