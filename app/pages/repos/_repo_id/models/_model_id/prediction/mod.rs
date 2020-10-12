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
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use std::convert::TryInto;
use tangram_util::id::Id;

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
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
enum Prediction {
	Regression(RegressionPrediction),
	Classification(ClassificationPrediction),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RegressionPrediction {
	value: f32,
	shap_chart_data: ShapChartData,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ClassificationPrediction {
	class_name: String,
	probability: f32,
	probabilities: Vec<(String, f32)>,
	shap_chart_data: ShapChartData,
}

type ShapChartData = Vec<ShapChartSeries>;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ShapChartSeries {
	baseline: f32,
	baseline_label: String,
	label: String,
	output: f32,
	output_label: String,
	values: Vec<ShapChartValue>,
}

#[derive(serde::Serialize, Debug)]
struct ShapChartValue {
	feature: String,
	value: f32,
}

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
					.unwrap_or_else(|| "".to_owned());
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
					.unwrap_or_else(|| "".to_owned());
				Column::Text(Text { name, value })
			}
		})
		.collect();
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
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
	let output = tangram_core::predict::predict(&predict_model, examples, None);
	let predict_output: Prediction = match output {
		tangram_core::predict::PredictOutput::Regression(mut output) => {
			let output = output.remove(0);
			let shap_data = output.feature_contributions.unwrap();
			let prediction = RegressionPrediction {
				shap_chart_data: vec![ShapChartSeries {
					baseline: shap_data.baseline_value,
					baseline_label: format!("{}", shap_data.baseline_value),
					label: "output".to_owned(),
					output: shap_data.output_value,
					output_label: format!("{}", shap_data.output_value),
					values: shap_data
						.feature_contributions
						.into_iter()
						.map(compute_shap_chart_value)
						.collect(),
				}],
				value: output.value,
			};
			Prediction::Regression(prediction)
		}
		tangram_core::predict::PredictOutput::Classification(mut output) => {
			let output = output.remove(0);
			let shap_chart_data = output
				.feature_contributions
				.unwrap()
				.into_iter()
				.map(|(class, shap_data)| ShapChartSeries {
					baseline: shap_data.baseline_value,
					baseline_label: format!("{}", shap_data.baseline_value),
					label: class,
					output: shap_data.output_value,
					output_label: format!("{}", shap_data.output_value),
					values: shap_data
						.feature_contributions
						.into_iter()
						.map(compute_shap_chart_value)
						.collect(),
				})
				.collect::<Vec<_>>();
			let prediction = ClassificationPrediction {
				class_name: output.class_name,
				probability: output.probability,
				probabilities: output.probabilities.into_iter().collect(),
				shap_chart_data,
			};
			Prediction::Classification(prediction)
		}
	};
	predict_output
}

fn compute_shap_chart_value(
	feature_contribution: tangram_core::predict::FeatureContribution,
) -> ShapChartValue {
	match feature_contribution {
		tangram_core::predict::FeatureContribution::Identity {
			column_name,
			feature_contribution_value,
		} => ShapChartValue {
			feature: column_name,
			value: feature_contribution_value,
		},
		tangram_core::predict::FeatureContribution::Normalized {
			column_name,
			feature_contribution_value,
		} => ShapChartValue {
			feature: column_name,
			value: feature_contribution_value,
		},
		tangram_core::predict::FeatureContribution::OneHotEncoded {
			column_name,
			option,
			feature_value,
			feature_contribution_value,
		} => {
			let predicate = if feature_value { "is" } else { "is not" };
			let option = option
				.map(|option| format!("\"{}\"", option))
				.unwrap_or_else(|| "invalid".to_owned());
			let feature = format!("{} {} {}", column_name, predicate, option);
			ShapChartValue {
				feature,
				value: feature_contribution_value,
			}
		}
		tangram_core::predict::FeatureContribution::BagOfWords {
			column_name,
			token,
			feature_value,
			feature_contribution_value,
		} => {
			let predicate = if feature_value {
				"contains"
			} else {
				"does not contain"
			};
			let feature = format!("{} {} \"{}\"", column_name, predicate, token);
			ShapChartValue {
				feature,
				value: feature_contribution_value,
			}
		}
	}
}
