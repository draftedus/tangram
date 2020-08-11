#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
	pub id: String,
	pub name: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutProps {
	pub id: String,
	pub title: String,
	pub models: Vec<RepoModel>,
	pub owner_name: String,
	pub owner_url: String,
	pub model_id: String,
	pub model_title: String,
	pub selected_item: ModelSideNavItem,
}

#[derive(serde::Serialize, Clone, Debug)]
pub enum ModelSideNavItem {
	#[serde(rename = "overview")]
	Overview,
	#[serde(rename = "training_stats")]
	TrainingStats,
	#[serde(rename = "training_metrics")]
	TrainingMetrics,
	#[serde(rename = "introspection")]
	Introspection,
	#[serde(rename = "predict")]
	Predict,
	#[serde(rename = "tuning")]
	Tuning,
	#[serde(rename = "production_stats")]
	ProductionStats,
	#[serde(rename = "production_metrics")]
	ProductionMetrics,
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
