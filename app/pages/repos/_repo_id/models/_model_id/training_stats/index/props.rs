use crate::layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub column_count: usize,
	pub column_stats: Vec<ColumnStats>,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub target_column_stats: ColumnStats,
	pub row_count: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnStats {
	pub invalid_count: Option<usize>,
	pub max: Option<f32>,
	pub mean: Option<f32>,
	pub min: Option<f32>,
	pub name: String,
	pub std: Option<f32>,
	pub column_type: ColumnType,
	pub unique_count: Option<usize>,
	pub variance: Option<f32>,
}

#[derive(serde::Serialize)]
pub enum ColumnType {
	#[serde(rename = "unknown")]
	Unknown,
	#[serde(rename = "number")]
	Number,
	#[serde(rename = "enum")]
	Enum,
	#[serde(rename = "text")]
	Text,
}
