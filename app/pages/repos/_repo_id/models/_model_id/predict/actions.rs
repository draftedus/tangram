use crate::app::{
	error::Error,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::Neg;
use tangram::{id::Id, types, *};

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
pub enum Action {
	#[serde(rename = "predict")]
	Predict(PredictAction),
}

#[derive(serde::Deserialize, Debug)]
pub struct PredictAction {
	pub examples: tangram::predict::PredictInput,
	pub options: Option<tangram::predict::PredictOptions>,
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

pub async fn actions(
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
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let request: PredictAction =
		serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	// get the necessary data from the model
	let rows = db
		.query(
			"
				select
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	db.commit().await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let bytes: Vec<u8> = row.get(0);
	let model = types::Model::from_slice(bytes.as_slice()).unwrap();
	let predict_model: predict::PredictModel = model.try_into().unwrap();
	let output = tangram::predict::predict(&predict_model, request.examples, request.options);
	let output: PredictOutput = match output {
		tangram::predict::PredictOutput::Classification(output) => {
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
		tangram::predict::PredictOutput::Regression(output) => PredictOutput::Regression(
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
