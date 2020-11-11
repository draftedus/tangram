use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(serde::Serialize)]
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
	pub histogram: Option<Vec<(String, u64)>>,
	pub invalid_count: u64,
	pub name: String,
	pub unique_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
	pub name: String,
	pub tokens: Vec<TokenStats>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
	pub token: String,
	pub count: u64,
	pub examples_count: u64,
}
