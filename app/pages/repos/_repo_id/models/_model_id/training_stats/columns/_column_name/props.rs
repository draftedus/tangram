use super::enum_column::EnumColumn;
use html::{component, html};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Number {
	pub invalid_count: u64,
	pub max: f32,
	pub mean: f32,
	pub min: f32,
	pub name: String,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub std: f32,
	pub unique_count: u64,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
	pub histogram: Option<Vec<(String, u64)>>,
	pub invalid_count: u64,
	pub name: String,
	pub unique_count: u64,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Text {
	pub name: String,
	pub n_tokens: usize,
	pub tokens: Vec<TokenStats>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
	pub token: String,
	pub count: u64,
	pub examples_count: u64,
}

#[component]
pub fn Page(inner: Inner, model_layout_info: ModelLayoutInfo, page_info: PageInfo) {
	let inner = match inner {
		Inner::Number(_) => todo!(),
		Inner::Enum(inner) => html! {
			<EnumColumn props={inner} />
		},
		Inner::Text(_) => todo!(),
	};
	html! {
		<ModelLayout
			info={model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingStats}
		>
			{inner}
		</ModelLayout>
	}
}
