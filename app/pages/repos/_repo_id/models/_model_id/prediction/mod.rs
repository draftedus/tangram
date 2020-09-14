use crate::{
	common::{
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::Neg;
use tangram_id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, search_params).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/prediction", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	columns: Vec<Column>,
	id: String,
	model_layout_info: ModelLayoutInfo,
	prediction: Option<Prediction>,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Column {
	Unknown(Unknown),
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Unknown {
	name: String,
	value: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Number {
	name: String,
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
	value: String,
}

#[derive(serde::Serialize)]
struct Enum {
	name: String,
	options: Vec<String>,
	value: String,
	histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Text {
	name: String,
	value: String,
}

#[derive(serde::Serialize, Debug)]
pub struct PredictResponse {
	pub output: Prediction,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum Prediction {
	Regression(RegressionPredictOutput),
	Classification(ClassificationPrediction),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictOutput {
	pub value: f32,
	pub shap_chart_data: Vec<RegressionShapValuesOutput>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPrediction {
	pub class_name: String,
	pub probabilities: Vec<(String, f32)>,
	pub probability: f32,
	pub shap_chart_data: Vec<ClassificationShapValuesOutput>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationShapValuesOutput {
	pub label: String,
	pub baseline: f32,
	pub baseline_label: String,
	pub baseline_probability: f32,
	pub output: f32,
	pub output_label: String,
	pub values: Vec<ShapValue>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionShapValuesOutput {
	pub baseline: f32,
	pub baseline_label: String,
	pub label: String,
	pub output: f32,
	pub output_label: String,
	pub values: Vec<ShapValue>,
}

#[derive(serde::Serialize, Debug)]
pub struct ShapValue {
	feature: String,
	value: f32,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
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
	let column_stats = match &model {
		tangram_core::model::Model::Classifier(model) => &model.overall_column_stats,
		tangram_core::model::Model::Regressor(model) => &model.overall_column_stats,
	};
	let columns: Vec<Column> = column_stats
		.iter()
		.map(|column_stats| match column_stats {
			tangram_core::model::ColumnStats::Unknown(column_stats) => {
				let name = column_stats.column_name.to_owned();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.map(|s| s.to_owned())
					.unwrap_or_else(|| "".to_string());
				Column::Unknown(Unknown { name, value })
			}
			tangram_core::model::ColumnStats::Number(column_stats) => {
				let name = column_stats.column_name.to_owned();
				let mean = column_stats.mean;
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.map(|s| s.to_owned())
					.unwrap_or_else(|| mean.to_string());
				Column::Number(Number {
					name,
					max: column_stats.max,
					min: column_stats.min,
					p25: column_stats.p25,
					p50: column_stats.p50,
					p75: column_stats.p75,
					value,
				})
			}
			tangram_core::model::ColumnStats::Enum(column_stats) => {
				let histogram = &column_stats.histogram;
				let options = histogram.iter().map(|(key, _)| key.to_owned()).collect();
				let name = column_stats.column_name.to_owned();
				let mode: String = column_stats
					.histogram
					.iter()
					.max_by(|a, b| a.1.cmp(&b.1))
					.unwrap()
					.0
					.to_owned();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.map(|s| s.to_owned())
					.unwrap_or(mode);
				let histogram = column_stats.histogram.to_owned();
				Column::Enum(Enum {
					name,
					options,
					value,
					histogram,
				})
			}
			tangram_core::model::ColumnStats::Text(column_stats) => {
				let name = column_stats.column_name.to_owned();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.map(|s| s.to_owned())
					.unwrap_or_else(|| "".to_string());
				Column::Text(Text { name, value })
			}
		})
		.collect();
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;

	// fill in prediction information
	let prediction = if let Some(search_params) = search_params {
		Some(predict(model, columns.as_slice(), search_params))
	} else {
		None
	};

	db.commit().await?;
	Ok(Props {
		model_layout_info,
		id: model_id.to_string(),
		columns,
		prediction,
	})
}

fn predict(
	model: tangram_core::model::Model,
	columns: &[Column],
	search_params: BTreeMap<String, String>,
) -> Prediction {
	let mut column_lookup = BTreeMap::new();
	for column in columns.iter() {
		match column {
			Column::Number(number_column) => {
				column_lookup.insert(number_column.name.to_owned(), column);
			}
			Column::Enum(enum_column) => {
				column_lookup.insert(enum_column.name.to_owned(), column);
			}
			Column::Text(text_column) => {
				column_lookup.insert(text_column.name.to_owned(), column);
			}
			_ => unreachable!(),
		}
	}
	let predict_model: tangram_core::predict::PredictModel = model.try_into().unwrap();
	let mut example: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
	for (key, value) in search_params.into_iter() {
		match column_lookup.get(&key) {
			Some(column) => match column {
				Column::Text(_) => {
					example.insert(key, serde_json::Value::String(value));
				}
				Column::Enum(_) => {
					example.insert(key, serde_json::Value::String(value));
				}
				Column::Number(_) => {
					if value == "" {
						continue;
					}
					let value = match lexical::parse::<f64, _>(value) {
						Ok(n) => n,
						Err(_) => {
							panic!();
						}
					};
					example.insert(
						key,
						serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()),
					);
				}
				_ => unreachable!(),
			},
			None => panic!(),
		}
	}
	let examples = tangram_core::predict::PredictInput(vec![example]);
	let options = tangram_core::predict::PredictOptions { threshold: 0.5 };
	let output = tangram_core::predict::predict(&predict_model, examples, Some(options));
	let predict_output: Prediction = match output {
		tangram_core::predict::PredictOutput::Classification(mut output) => {
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
				tangram_core::predict::PredictModel::LinearBinaryClassifier(_) => Box::new(sigmoid),
				tangram_core::predict::PredictModel::TreeBinaryClassifier(_) => Box::new(sigmoid),
				tangram_core::predict::PredictModel::LinearMulticlassClassifier(_) => {
					Box::new(softmax)
				}
				tangram_core::predict::PredictModel::TreeMulticlassClassifier(_) => {
					Box::new(softmax)
				}
				_ => unreachable!(),
			};
			let output = output.remove(0);
			let output_probabilities = output.probabilities;
			let shap_values = output.shap_values.unwrap();
			let baselines = shap_values
				.iter()
				.map(|(_, shap_values)| shap_values.baseline)
				.collect::<Vec<f32>>();
			let baseline_probabilities = get_baseline_probabilities(baselines.as_slice());
			let probability = output_probabilities
				.get(&output.class_name)
				.unwrap()
				.to_owned();
			let shap_chart_data = shap_values
				.into_iter()
				.zip(baseline_probabilities)
				.map(|((class, shap_values), baseline_probability)| {
					let output_probability = output_probabilities.get(&class).unwrap();
					let output = shap_values.baseline
						+ shap_values.values.iter().fold(0.0, |mut sum, shap_value| {
							sum += shap_value.1;
							sum
						});
					ClassificationShapValuesOutput {
						baseline: shap_values.baseline,
						baseline_probability,
						baseline_label: format!("{:.2}%", baseline_probability * 100.0),
						output,
						label: class.to_string(),
						output_label: format!("{:.2}%", output_probability * 100.0),
						values: shap_values
							.values
							.into_iter()
							.map(|(feature, value)| ShapValue { feature, value })
							.collect(),
					}
				})
				.collect::<Vec<_>>();
			let prediction = ClassificationPrediction {
				class_name: output.class_name,
				probability,
				probabilities: output_probabilities.into_iter().collect(),
				shap_chart_data,
			};
			Prediction::Classification(prediction)
		}
		tangram_core::predict::PredictOutput::Regression(mut output) => {
			let output = output.remove(0);
			let shap_values = output.shap_values.unwrap();
			let prediction = RegressionPredictOutput {
				shap_chart_data: vec![RegressionShapValuesOutput {
					baseline: shap_values.baseline,
					baseline_label: shap_values.baseline.to_string(),
					label: "output".to_string(),
					output: output.value,
					output_label: output.value.to_string(),
					values: shap_values
						.values
						.into_iter()
						.map(|(feature, value)| ShapValue { feature, value })
						.collect(),
				}],
				value: output.value,
			};
			Prediction::Regression(prediction)
		}
	};
	predict_output
}
