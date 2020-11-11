use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub model_layout_info: ModelLayoutInfo,
	pub prediction_table: Option<PredictionTable>,
	pub pagination: Pagination,
}

#[derive(serde::Serialize, Debug)]
pub struct PredictionTable {
	pub rows: Vec<PredictionTableRow>,
}

#[derive(serde::Serialize, Debug)]
pub struct PredictionTableRow {
	pub date: String,
	pub identifier: String,
	pub output: String,
}

#[derive(serde::Serialize, Debug)]
pub struct Pagination {
	pub after: Option<usize>,
	pub before: Option<usize>,
}

#[derive(serde::Serialize, Debug)]
pub struct PaginationRange {
	pub start: usize,
	pub end: usize,
	pub total: usize,
}
