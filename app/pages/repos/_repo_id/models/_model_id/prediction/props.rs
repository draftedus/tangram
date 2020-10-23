use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		predict::{ColumnType, InputTable, InputTableRow, Prediction, PredictionResult},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use std::collections::BTreeMap;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	id: String,
	model_layout_info: ModelLayoutInfo,
	inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	PredictionForm(PredictionForm),
	PredictionResult(PredictionResult),
}

#[derive(serde::Serialize)]
pub struct PredictionForm {
	form: PredictForm,
}

#[derive(serde::Serialize)]
pub struct PredictForm {
	fields: Vec<Column>,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
pub enum Column {
	#[serde(rename = "unknown")]
	Unknown(Unknown),
	#[serde(rename = "number")]
	Number(Number),
	#[serde(rename = "enum")]
	Enum(Enum),
	#[serde(rename = "text")]
	Text(Text),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unknown {
	name: String,
	value: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Number {
	name: String,
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
	value: String,
}

#[derive(serde::Serialize)]
pub struct Enum {
	name: String,
	options: Vec<String>,
	value: String,
	histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
	name: String,
	value: String,
}

pub async fn props(
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
		tangram_core::model::Model::Regressor(model) => &model.overall_column_stats,
		tangram_core::model::Model::BinaryClassifier(model) => &model.overall_column_stats,
		tangram_core::model::Model::MulticlassClassifier(model) => &model.overall_column_stats,
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
	let inner = if let Some(search_params) = search_params {
		let prediction = predict(model, columns.as_slice(), search_params);
		let input_table_rows = columns
			.into_iter()
			.map(|column| {
				let (column_type, column_name, value) = match column {
					Column::Unknown(column) => (ColumnType::Unknown, column.name, column.value),
					Column::Enum(column) => (ColumnType::Enum, column.name, column.value),
					Column::Number(column) => (ColumnType::Number, column.name, column.value),
					Column::Text(column) => (ColumnType::Text, column.name, column.value),
				};
				InputTableRow {
					column_name,
					value: serde_json::Value::String(value),
					column_type,
				}
			})
			.collect();
		Inner::PredictionResult(PredictionResult {
			input_table: InputTable {
				rows: input_table_rows,
			},
			prediction,
		})
	} else {
		Inner::PredictionForm(PredictionForm {
			form: PredictForm { fields: columns },
		})
	};

	db.commit().await?;

	Ok(Props {
		model_layout_info,
		id: model_id.to_string(),
		inner,
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
						Ok(value) => value,
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
	crate::common::predict::predict(model, example)
}
