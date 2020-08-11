use crate::{
	error::Error,
	helpers::{
		model::{get_model, Model},
		repos::get_model_layout_info,
	},
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/introspection", props)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	title: String,
	model_layout_info: types::ModelLayoutInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	LinearRegressor(LinearRegressor),
	LinearBinaryClassifier(LinearBinaryClassifier),
	LinearMulticlassClassifier(LinearMulticlassClassifier),
	GbtBinaryClassifier(GbtBinaryClassifier),
	GbtMulticlassClassifier(GbtMulticlassClassifier),
	GbtRegressor(GbtRegressor),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearRegressor {
	bias: f32,
	target_column_name: String,
	weights: Vec<(String, f32)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearBinaryClassifier {
	bias: f32,
	target_column_name: String,
	positive_class_name: String,
	weights: Vec<(String, f32)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinearMulticlassClassifier {
	biases: Vec<f32>,
	target_column_name: String,
	classes: Vec<String>,
	weights: Vec<Vec<(String, f32)>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GbtBinaryClassifier {
	feature_importances: Vec<(String, f32)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GbtMulticlassClassifier {
	feature_importances: Vec<(String, f32)>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GbtRegressor {
	feature_importances: Vec<(String, f32)>,
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let Model { id, title, data } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;
	let inner = match model {
		tangram_core::types::Model::Classifier(model) => {
			let target_column_name = model.target_column_name.as_option().unwrap().to_owned();
			let classes = model.classes().to_owned();
			match model.model.into_option().unwrap() {
				tangram_core::types::ClassificationModel::UnknownVariant(_, _, _) => {
					unimplemented!()
				}
				tangram_core::types::ClassificationModel::LinearBinary(inner_model) => {
					let feature_groups = inner_model.feature_groups.as_option().unwrap();
					let feature_names = compute_feature_names(feature_groups);
					let weights = inner_model.weights.as_option().unwrap();
					let mut weights = feature_names
						.into_iter()
						.zip(weights)
						.map(|(f, w)| (f, *w))
						.collect::<Vec<(String, f32)>>();
					weights.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::LinearBinaryClassifier(LinearBinaryClassifier {
						bias: *inner_model.bias.as_option().unwrap(),
						target_column_name,
						positive_class_name: classes[1].clone(),
						weights,
					})
				}
				tangram_core::types::ClassificationModel::LinearMulticlass(inner_model) => {
					let feature_groups = inner_model.feature_groups.as_option().unwrap();
					let n_classes = inner_model
						.n_classes
						.as_option()
						.unwrap()
						.to_usize()
						.unwrap();
					let n_features = inner_model
						.n_features
						.as_option()
						.unwrap()
						.to_usize()
						.unwrap();
					let weights = Array2::from_shape_vec(
						(n_classes, n_features),
						inner_model.weights.into_option().unwrap(),
					)
					.unwrap();
					let feature_names = compute_feature_names(feature_groups);
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
						biases: inner_model.biases.into_option().unwrap(),
						target_column_name,
						classes,
						weights,
					})
				}
				tangram_core::types::ClassificationModel::GbtBinary(inner_model) => {
					let feature_groups = inner_model.feature_groups.as_option().unwrap();
					let feature_importances = inner_model
						.feature_importances
						.as_option()
						.unwrap()
						.as_slice();
					let feature_names = compute_feature_names(feature_groups);
					let mut feature_importances: Vec<(String, f32)> = feature_names
						.into_iter()
						.zip(feature_importances)
						.map(|(f, w)| (f, *w))
						.collect();
					feature_importances
						.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::GbtBinaryClassifier(GbtBinaryClassifier {
						feature_importances,
					})
				}
				tangram_core::types::ClassificationModel::GbtMulticlass(inner_model) => {
					let feature_groups = inner_model.feature_groups.as_option().unwrap();
					let feature_importances = inner_model
						.feature_importances
						.as_option()
						.unwrap()
						.as_slice();
					let feature_names = compute_feature_names(feature_groups);
					let mut feature_importances: Vec<(String, f32)> = feature_names
						.into_iter()
						.zip(feature_importances)
						.map(|(f, w)| (f, *w))
						.collect();
					feature_importances
						.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
					Inner::GbtMulticlassClassifier(GbtMulticlassClassifier {
						feature_importances,
					})
				}
			}
		}
		tangram_core::types::Model::Regressor(model) => match model.model.as_option().unwrap() {
			tangram_core::types::RegressionModel::Linear(inner_model) => {
				let target_column_name = model.target_column_name.as_option().unwrap().to_owned();
				let feature_groups = inner_model.feature_groups.as_option().unwrap();
				let weights = inner_model.weights.as_option().unwrap();
				let feature_names = compute_feature_names(feature_groups);
				let mut weights = feature_names
					.into_iter()
					.zip(weights)
					.map(|(f, w)| (f, *w))
					.collect::<Vec<(String, f32)>>();
				weights.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::LinearRegressor(LinearRegressor {
					bias: *inner_model.bias.as_option().unwrap(),
					target_column_name,
					weights,
				})
			}
			tangram_core::types::RegressionModel::Gbt(inner_model) => {
				let feature_groups = inner_model.feature_groups.as_option().unwrap();
				let feature_importances = inner_model
					.feature_importances
					.as_option()
					.unwrap()
					.as_slice();
				let feature_names = compute_feature_names(feature_groups);
				let mut feature_importances: Vec<(String, f32)> = feature_names
					.into_iter()
					.zip(feature_importances)
					.map(|(f, w)| (f, *w))
					.collect();
				feature_importances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
				Inner::GbtRegressor(GbtRegressor {
					feature_importances,
				})
			}
			tangram_core::types::RegressionModel::UnknownVariant(_, _, _) => unimplemented!(),
		},
		_ => return Err(Error::BadRequest.into()),
	};

	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;

	db.commit().await?;

	Ok(Props {
		id: id.to_string(),
		title,
		inner,
		model_layout_info,
	})
}

fn compute_feature_names(feature_groups: &[tangram_core::types::FeatureGroup]) -> Vec<String> {
	feature_groups
		.iter()
		.flat_map(|feature_group| match feature_group {
			tangram_core::types::FeatureGroup::Identity(feature_group) => vec![feature_group
				.source_column_name
				.as_option()
				.unwrap()
				.to_owned()],
			tangram_core::types::FeatureGroup::Normalized(feature_group) => vec![feature_group
				.source_column_name
				.as_option()
				.unwrap()
				.to_owned()],
			tangram_core::types::FeatureGroup::OneHotEncoded(feature_group) => {
				vec!["OOV".to_string()]
					.iter()
					.chain(feature_group.categories.as_option().unwrap().iter())
					.map(|category| {
						format!(
							"{} = {}",
							feature_group
								.source_column_name
								.as_option()
								.unwrap()
								.to_owned(),
							category.to_owned()
						)
					})
					.collect()
			}
			tangram_core::types::FeatureGroup::BagOfWords(feature_group) => feature_group
				.tokens
				.as_option()
				.unwrap()
				.iter()
				.map(|(token, _)| {
					format!(
						"{} contains {}",
						feature_group
							.source_column_name
							.as_option()
							.unwrap()
							.to_owned(),
						token
					)
				})
				.collect(),
			tangram_core::types::FeatureGroup::UnknownVariant(_, _, _) => unimplemented!(),
		})
		.collect()
}
