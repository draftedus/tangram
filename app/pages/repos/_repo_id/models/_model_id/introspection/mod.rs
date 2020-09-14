use crate::{
	common::{
		model::{get_model, Model},
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_core::util::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/introspection", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	LinearRegressor(LinearRegressor),
	LinearBinaryClassifier(LinearBinaryClassifier),
	LinearMulticlassClassifier(LinearMulticlassClassifier),
	TreeBinaryClassifier(TreeBinaryClassifier),
	TreeMulticlassClassifier(TreeMulticlassClassifier),
	TreeRegressor(TreeRegressor),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearRegressor {
	bias: f32,
	target_column_name: String,
	weights: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearBinaryClassifier {
	bias: f32,
	target_column_name: String,
	positive_class_name: String,
	weights: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearMulticlassClassifier {
	biases: Vec<f32>,
	target_column_name: String,
	classes: Vec<String>,
	weights: Vec<Vec<(String, f32)>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeBinaryClassifier {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeMulticlassClassifier {
	feature_importances: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeRegressor {
	feature_importances: Vec<(String, f32)>,
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
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
	let Model { id, data } = get_model(&mut db, model_id).await?;
	let model = tangram_core::model::Model::from_slice(&data)?;
	let inner = match model {
		tangram_core::model::Model::Classifier(model) => {
			let target_column_name = model.target_column_name.to_owned();
			let classes = model.classes().to_owned();
			match model.model {
				tangram_core::model::ClassificationModel::LinearBinary(inner_model) => {
					let feature_groups = inner_model.feature_groups;
					let feature_names = compute_feature_names(&feature_groups);
					let weights = inner_model.weights;
					let mut weights = feature_names
						.into_iter()
						.zip(weights)
						.map(|(f, w)| (f, w))
						.collect::<Vec<(String, f32)>>();
					weights.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::LinearBinaryClassifier(LinearBinaryClassifier {
						bias: inner_model.bias,
						target_column_name,
						positive_class_name: classes[1].clone(),
						weights,
					})
				}
				tangram_core::model::ClassificationModel::LinearMulticlass(inner_model) => {
					let feature_groups = inner_model.feature_groups;
					let n_classes = inner_model.n_classes.to_usize().unwrap();
					let n_features = inner_model.n_features.to_usize().unwrap();
					let weights =
						Array2::from_shape_vec((n_classes, n_features), inner_model.weights)
							.unwrap();
					let feature_names = compute_feature_names(&feature_groups);
					let weights: Vec<Vec<(String, f32)>> = weights
						.axis_iter(Axis(0))
						.map(|weights| {
							let mut weights = feature_names
								.iter()
								.zip(weights)
								.map(|(f, w)| (f.to_owned(), *w))
								.collect::<Vec<(String, f32)>>();
							weights.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
							weights
						})
						.collect();
					Inner::LinearMulticlassClassifier(LinearMulticlassClassifier {
						biases: inner_model.biases,
						target_column_name,
						classes,
						weights,
					})
				}
				tangram_core::model::ClassificationModel::TreeBinary(inner_model) => {
					let feature_groups = inner_model.feature_groups;
					let feature_importances = inner_model.feature_importances.as_slice();
					let feature_names = compute_feature_names(&feature_groups);
					let mut feature_importances: Vec<(String, f32)> = feature_names
						.into_iter()
						.zip(feature_importances)
						.map(|(f, w)| (f, *w))
						.collect();
					feature_importances
						.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::TreeBinaryClassifier(TreeBinaryClassifier {
						feature_importances,
					})
				}
				tangram_core::model::ClassificationModel::TreeMulticlass(inner_model) => {
					let feature_groups = inner_model.feature_groups;
					let feature_importances = inner_model.feature_importances.as_slice();
					let feature_names = compute_feature_names(&feature_groups);
					let mut feature_importances: Vec<(String, f32)> = feature_names
						.into_iter()
						.zip(feature_importances)
						.map(|(f, w)| (f, *w))
						.collect();
					feature_importances
						.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::TreeMulticlassClassifier(TreeMulticlassClassifier {
						feature_importances,
					})
				}
			}
		}
		tangram_core::model::Model::Regressor(model) => match model.model {
			tangram_core::model::RegressionModel::Linear(inner_model) => {
				let target_column_name = model.target_column_name.to_owned();
				let feature_groups = inner_model.feature_groups;
				let weights = inner_model.weights;
				let feature_names = compute_feature_names(&feature_groups);
				let mut weights: Vec<(String, f32)> = feature_names
					.into_iter()
					.zip(weights)
					.map(|(f, w)| (f, w))
					.collect();
				weights.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::LinearRegressor(LinearRegressor {
					bias: inner_model.bias,
					target_column_name,
					weights,
				})
			}
			tangram_core::model::RegressionModel::Tree(inner_model) => {
				let feature_groups = inner_model.feature_groups;
				let feature_importances = inner_model.feature_importances.as_slice();
				let feature_names = compute_feature_names(&feature_groups);
				let mut feature_importances: Vec<(String, f32)> = feature_names
					.into_iter()
					.zip(feature_importances)
					.map(|(f, w)| (f, *w))
					.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::TreeRegressor(TreeRegressor {
					feature_importances,
				})
			}
		},
	};

	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;

	db.commit().await?;

	Ok(Props {
		id: id.to_string(),
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
				vec!["OOV".to_string()]
					.iter()
					.chain(feature_group.categories.iter())
					.map(|category| {
						format!(
							"{} = {}",
							feature_group.source_column_name.to_owned(),
							category.to_owned()
						)
					})
					.collect()
			}
			tangram_core::model::FeatureGroup::BagOfWords(feature_group) => feature_group
				.tokens
				.iter()
				.map(|(token, _)| {
					format!(
						"{} contains {}",
						feature_group.source_column_name.to_owned(),
						token
					)
				})
				.collect(),
		})
		.collect()
}
