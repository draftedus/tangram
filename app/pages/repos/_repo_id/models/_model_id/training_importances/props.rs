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
	LinearRegressor(LinearRegressorProps),
	TreeRegressor(TreeRegressorProps),
	LinearBinaryClassifier(LinearBinaryClassifierProps),
	TreeBinaryClassifier(TreeBinaryClassifierProps),
	LinearMulticlassClassifier(LinearMulticlassClassifierProps),
	TreeMulticlassClassifier(TreeMulticlassClassifierProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureImportance {
	pub feature_importance_value: f32,
	pub feature_name: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearRegressorProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeRegressorProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearBinaryClassifierProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeBinaryClassifierProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearMulticlassClassifierProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeMulticlassClassifierProps {
	pub feature_importances: Vec<FeatureImportance>,
	pub n_features: usize,
}
