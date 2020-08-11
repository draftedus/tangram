use crate::{
	error::Error,
	helpers::{
		model::{get_model, Model},
		repos::get_model_layout_props,
	},
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::Neg;
use tangram_core::{id::Id, *};

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/predict", props)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	columns: Vec<Column>,
	title: String,
	id: String,
	model_layout_props: types::ModelLayoutProps,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Column {
	Unknown(Unknown),
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Unknown {
	name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Number {
	name: String,
	max: f32,
	min: f32,
}

#[derive(Serialize)]
struct Enum {
	name: String,
	options: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Text {
	name: String,
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
	let column_stats = match model {
		tangram_core::types::Model::Classifier(model) => {
			model.overall_column_stats.into_option().unwrap()
		}
		tangram_core::types::Model::Regressor(model) => {
			model.overall_column_stats.into_option().unwrap()
		}
		_ => return Err(Error::BadRequest.into()),
	};
	let columns = column_stats
		.into_iter()
		.map(|column_stats| match column_stats {
			tangram_core::types::ColumnStats::Unknown(column_stats) => Column::Unknown(Unknown {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
			}),
			tangram_core::types::ColumnStats::Number(column_stats) => Column::Number(Number {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
				max: *column_stats.max.as_option().unwrap(),
				min: *column_stats.min.as_option().unwrap(),
			}),
			tangram_core::types::ColumnStats::Enum(column_stats) => {
				let histogram = column_stats.histogram.as_option().unwrap();
				let options = histogram.iter().map(|(key, _)| key.to_owned()).collect();
				Column::Enum(Enum {
					name: column_stats.column_name.as_option().unwrap().to_owned(),
					options,
				})
			}
			tangram_core::types::ColumnStats::Text(column_stats) => Column::Text(Text {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
			}),
			tangram_core::types::ColumnStats::UnknownVariant(_, _, _) => unimplemented!(),
		})
		.collect();
	let model_layout_props =
		get_model_layout_props(&mut db, id, types::ModelSideNavItem::Predict).await?;
	db.commit().await?;
	Ok(Props {
		model_layout_props,
		id: id.to_string(),
		title,
		columns,
	})
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
pub enum Action {
	#[serde(rename = "predict")]
	Predict(PredictAction),
}

#[derive(serde::Deserialize, Debug)]
pub struct PredictAction {
	pub examples: tangram_core::predict::PredictInput,
	pub options: Option<tangram_core::predict::PredictOptions>,
}

#[derive(serde::Serialize, Debug)]
pub struct PredictResponse {
	pub output: PredictOutput,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum PredictOutput {
	Regression(Vec<RegressionPredictOutput>),
	Classification(Vec<ClassificationPredictOutput>),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictOutput {
	pub value: f32,
	pub shap_values: Option<RegressionShapValuesOutput>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictOutput {
	pub class_name: String,
	pub probabilities: BTreeMap<String, f32>,
	pub shap_values: Option<BTreeMap<String, ClassificationShapValuesOutput>>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationShapValuesOutput {
	pub baseline: f32,
	pub baseline_probability: f32,
	pub output: f32,
	pub values: Vec<(String, f32)>,
}

#[derive(serde::Serialize, Debug)]
pub struct RegressionShapValuesOutput {
	pub baseline: f32,
	pub values: Vec<(String, f32)>,
}

pub async fn post(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	predict(request, context, model_id).await
}

async fn predict(
	mut request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
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
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let request: PredictAction =
		serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;

	let Model { data, .. } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(data.as_slice()).unwrap();

	let predict_model: predict::PredictModel = model.try_into().unwrap();
	let output = tangram_core::predict::predict(&predict_model, request.examples, request.options);
	let output: PredictOutput = match output {
		tangram_core::predict::PredictOutput::Classification(output) => {
			// get baseline probabliities
			let softmax = |logits: &[f32]| {
				let mut probabilities = logits.to_owned();
				let max = probabilities.iter().fold(std::f32::MIN, |a, &b| a.max(b));
				probabilities.iter_mut().for_each(|p| *p -= max);
				probabilities.iter_mut().for_each(|l| *l = l.exp());
				let sum = probabilities.iter().fold(0.0, |a, b| a + b);
				probabilities.iter_mut().for_each(|p| *p /= sum);
				probabilities
			};
			let sigmoid = |logits: &[f32]| {
				let logit = logits[0];
				vec![1.0 / (logit.neg().exp() + 1.0)]
			};
			let get_baseline_probabilities: Box<dyn Fn(&[f32]) -> Vec<f32>> = match predict_model {
				predict::PredictModel::LinearBinaryClassifier(_) => Box::new(sigmoid),
				predict::PredictModel::GbtBinaryClassifier(_) => Box::new(sigmoid),
				predict::PredictModel::LinearMulticlassClassifier(_) => Box::new(softmax),
				predict::PredictModel::GbtMulticlassClassifier(_) => Box::new(softmax),
				_ => unreachable!(),
			};
			PredictOutput::Classification(
				output
					.into_iter()
					.map(|output| {
						let shap_values = output.shap_values.unwrap();
						let baselines = shap_values
							.iter()
							.map(|(_, shap_values)| shap_values.baseline)
							.collect::<Vec<f32>>();
						let baseline_probabilities =
							get_baseline_probabilities(baselines.as_slice());
						ClassificationPredictOutput {
							class_name: output.class_name,
							probabilities: output.probabilities,
							shap_values: Some(
								shap_values
									.into_iter()
									.zip(baseline_probabilities)
									.map(|((class, shap_values), baseline_probability)| {
										let output = shap_values.baseline
											+ shap_values.values.iter().fold(
												0.0,
												|mut sum, shap_value| {
													sum += shap_value.1;
													sum
												},
											);
										(
											class,
											ClassificationShapValuesOutput {
												baseline: shap_values.baseline,
												baseline_probability,
												output,
												values: shap_values.values,
											},
										)
									})
									.collect::<BTreeMap<String, ClassificationShapValuesOutput>>(),
							),
						}
					})
					.collect(),
			)
		}
		tangram_core::predict::PredictOutput::Regression(output) => PredictOutput::Regression(
			output
				.into_iter()
				.map(|output| {
					let shap_values = output.shap_values.unwrap();
					RegressionPredictOutput {
						shap_values: Some(RegressionShapValuesOutput {
							baseline: shap_values.baseline,
							values: shap_values.values,
						}),
						value: output.value,
					}
				})
				.collect(),
		),
	};
	let response = PredictResponse { output };
	let response = serde_json::to_vec(&response)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}
