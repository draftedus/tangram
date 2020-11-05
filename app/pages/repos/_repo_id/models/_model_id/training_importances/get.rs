use super::props::{
	FeatureImportance, Inner, LinearBinaryClassifierProps, LinearMulticlassClassifierProps,
	LinearRegressorProps, Props, TreeBinaryClassifierProps, TreeMulticlassClassifierProps,
	TreeRegressorProps,
};
use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::{error::Result, id::Id, zip};

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_importances",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn props(context: &Context, request: Request<Body>, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model = get_model(&mut db, model_id).await?;
	let inner = match model {
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
				Inner::LinearRegressor(LinearRegressorProps {
					feature_importances,
				})
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
				Inner::TreeRegressor(TreeRegressorProps {
					feature_importances,
				})
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
				Inner::LinearBinaryClassifier(LinearBinaryClassifierProps {
					feature_importances,
				})
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
				Inner::TreeBinaryClassifier(TreeBinaryClassifierProps {
					feature_importances,
				})
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
				Inner::LinearMulticlassClassifier(LinearMulticlassClassifierProps {
					feature_importances,
				})
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
				Inner::TreeMulticlassClassifier(TreeMulticlassClassifierProps {
					feature_importances,
				})
			}
		},
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
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
