use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	// pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	#[serde(rename = "regressor")]
	Regressor(RegressorProps),
	#[serde(rename = "binary_classifier")]
	BinaryClassifier(BinaryClassifierProps),
	#[serde(rename = "multiclass_classifier")]
	MulticlassClassifier(MulticlassClassifierProps),
}

#[derive(serde::Serialize)]
pub struct RegressorProps {}

#[derive(serde::Serialize)]
pub struct BinaryClassifierProps {}

#[derive(serde::Serialize)]
pub struct MulticlassClassifierProps {}
