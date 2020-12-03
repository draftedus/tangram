use crate::page::{render, FeatureImportance, Props};
use pinwheel::client;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::{http, hyper};
use tangram_util::{error::Result, id::Id, zip};

const MAX_FEATURE_IMPORTANCES: usize = 1_000;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model = get_model(&mut db, model_id).await?;
	let (feature_importances, n_features) = match model {
		tangram_core::model::Model::Regressor(model) => match model.model {
			tangram_core::model::RegressionModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
			tangram_core::model::RegressionModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
		},
		tangram_core::model::Model::BinaryClassifier(model) => match &model.model {
			tangram_core::model::BinaryClassificationModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
			tangram_core::model::BinaryClassificationModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
		},
		tangram_core::model::Model::MulticlassClassifier(model) => match model.model {
			tangram_core::model::MulticlassClassificationModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
			tangram_core::model::MulticlassClassificationModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances =
					zip!(feature_names, inner_model.feature_importances.iter())
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_name,
								feature_importance_value: *feature_importance_value,
							},
						)
						.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				feature_importances.truncate(MAX_FEATURE_IMPORTANCES);
				(feature_importances, n_features)
			}
		},
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let props = Props {
		id: model_id.to_string(),
		feature_importances,
		n_features,
		model_layout_info,
	};
	db.commit().await?;
	let page_info = PageInfo {
		client_wasm_js_src: Some(client!()),
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn compute_feature_names(feature_groups: &[tangram_core::model::FeatureGroup]) -> Vec<String> {
	feature_groups
		.iter()
		.flat_map(|feature_group| match feature_group {
			tangram_core::model::FeatureGroup::Identity(feature_group) => {
				vec![feature_group.source_column_name.clone()]
			}
			tangram_core::model::FeatureGroup::Normalized(feature_group) => {
				vec![feature_group.source_column_name.clone()]
			}
			tangram_core::model::FeatureGroup::OneHotEncoded(feature_group) => {
				vec!["OOV".to_owned()]
					.iter()
					.chain(feature_group.options.iter())
					.map(|option| {
						format!(
							"{} = {}",
							feature_group.source_column_name.clone(),
							option.clone(),
						)
					})
					.collect()
			}
			tangram_core::model::FeatureGroup::BagOfWords(feature_group) => feature_group
				.tokens
				.iter()
				.map(|token| {
					format!(
						"{} contains {}",
						feature_group.source_column_name.clone(),
						token.token,
					)
				})
				.collect(),
		})
		.collect()
}
