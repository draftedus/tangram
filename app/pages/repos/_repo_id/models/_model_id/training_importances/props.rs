use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use itertools::izip;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
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
pub struct LinearRegressorProps {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeRegressorProps {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearBinaryClassifierProps {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeBinaryClassifierProps {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearMulticlassClassifierProps {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeMulticlassClassifierProps {
	feature_importances: Vec<(String, f32)>,
}

pub async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let model = get_model(&mut db, model_id).await?;
	let inner = match model {
		tangram_core::model::Model::Regressor(model) => match model.model {
			tangram_core::model::RegressionModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::LinearRegressor(LinearRegressorProps {
					feature_importances,
				})
			}
			tangram_core::model::RegressionModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::TreeRegressor(TreeRegressorProps {
					feature_importances,
				})
			}
		},
		tangram_core::model::Model::BinaryClassifier(model) => match &model.model {
			tangram_core::model::BinaryClassificationModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::LinearBinaryClassifier(LinearBinaryClassifierProps {
					feature_importances,
				})
			}
			tangram_core::model::BinaryClassificationModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::TreeBinaryClassifier(TreeBinaryClassifierProps {
					feature_importances,
				})
			}
		},
		tangram_core::model::Model::MulticlassClassifier(model) => match model.model {
			tangram_core::model::MulticlassClassificationModel::Linear(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::LinearMulticlassClassifier(LinearMulticlassClassifierProps {
					feature_importances,
				})
			}
			tangram_core::model::MulticlassClassificationModel::Tree(inner_model) => {
				let feature_names = compute_feature_names(&inner_model.feature_groups);
				let mut feature_importances: Vec<(String, f32)> =
					izip!(feature_names, inner_model.feature_importances.iter())
						.map(|(feature_name, feature_importance)| {
							(feature_name, *feature_importance)
						})
						.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::TreeMulticlassClassifier(TreeMulticlassClassifierProps {
					feature_importances,
				})
			}
		},
	};
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
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
				vec![feature_group.source_column_name.to_owned()]
			}
			tangram_core::model::FeatureGroup::Normalized(feature_group) => {
				vec![feature_group.source_column_name.to_owned()]
			}
			tangram_core::model::FeatureGroup::OneHotEncoded(feature_group) => {
				vec!["OOV".to_owned()]
					.iter()
					.chain(feature_group.options.iter())
					.map(|option| {
						format!(
							"{} = {}",
							feature_group.source_column_name.to_owned(),
							option.to_owned(),
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
						feature_group.source_column_name.to_owned(),
						token.token,
					)
				})
				.collect(),
		})
		.collect()
}
