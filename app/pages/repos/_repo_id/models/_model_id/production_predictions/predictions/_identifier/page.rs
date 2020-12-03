use tangram_app_common::predict::{InputTable, Prediction};
use tangram_app_layouts::{document::PageInfo, model_layout::ModelLayoutInfo};

#[derive(Clone)]
pub struct Props {
	pub model_layout_info: ModelLayoutInfo,
	pub identifier: String,
	pub inner: Inner,
}

#[derive(Clone)]
pub enum Inner {
	NotFound,
	Found(Found),
}

#[derive(Clone)]
pub struct Found {
	pub date: String,
	pub input_table: InputTable,
	pub prediction: Prediction,
}

pub fn render(_props: Props, _page_info: PageInfo) -> String {
	todo!()
}
