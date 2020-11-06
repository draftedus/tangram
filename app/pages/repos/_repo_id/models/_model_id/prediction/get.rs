use super::props::{
	Column, Enum, Inner, Number, PredictForm, PredictionForm, Props, Text, Unknown,
};
use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
		model::get_model,
		predict::{ColumnType, InputTable, InputTableRow, Prediction, PredictionResult},
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_util::id::Id;
use tangram_util::{err, error::Result};

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
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
	let column_stats = match &model {
		tangram_core::model::Model::Regressor(model) => &model.overall_column_stats,
		tangram_core::model::Model::BinaryClassifier(model) => &model.overall_column_stats,
		tangram_core::model::Model::MulticlassClassifier(model) => &model.overall_column_stats,
	};
	let columns: Vec<Column> = column_stats
		.iter()
		.map(|column_stats| match column_stats {
			tangram_core::model::ColumnStats::Unknown(column_stats) => {
				let name = column_stats.column_name.clone();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.cloned()
					.unwrap_or_else(|| "".to_owned());
				Column::Unknown(Unknown { name, value })
			}
			tangram_core::model::ColumnStats::Number(column_stats) => {
				let name = column_stats.column_name.clone();
				let mean = column_stats.mean;
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.cloned()
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
				let options = histogram.iter().map(|(key, _)| key.clone()).collect();
				let name = column_stats.column_name.clone();
				let mode: String = column_stats
					.histogram
					.iter()
					.max_by(|a, b| a.1.cmp(&b.1))
					.unwrap()
					.0
					.clone();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.cloned()
					.unwrap_or(mode);
				let histogram = column_stats.histogram.clone();
				Column::Enum(Enum {
					name,
					options,
					value,
					histogram,
				})
			}
			tangram_core::model::ColumnStats::Text(column_stats) => {
				let name = column_stats.column_name.clone();
				let value = search_params
					.as_ref()
					.and_then(|s| s.get(&name))
					.cloned()
					.unwrap_or_else(|| "".to_owned());
				Column::Text(Text { name, value })
			}
		})
		.collect();
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let inner = if let Some(search_params) = search_params {
		let prediction = predict(model, columns.as_slice(), search_params)?;
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
	let props = Props {
		model_layout_info,
		id: model_id.to_string(),
		inner,
	};
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/prediction", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

fn predict(
	model: tangram_core::model::Model,
	columns: &[Column],
	search_params: BTreeMap<String, String>,
) -> Result<Prediction> {
	let mut column_lookup = BTreeMap::new();
	for column in columns.iter() {
		match column {
			Column::Number(number_column) => {
				column_lookup.insert(number_column.name.clone(), column);
			}
			Column::Enum(enum_column) => {
				column_lookup.insert(enum_column.name.clone(), column);
			}
			Column::Text(text_column) => {
				column_lookup.insert(text_column.name.clone(), column);
			}
			_ => unreachable!(),
		}
	}
	let mut example: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
	for (key, value) in search_params.into_iter() {
		let column = match column_lookup.get(&key) {
			Some(column) => column,
			None => continue,
		};
		match column {
			Column::Unknown(_) => {
				return Err(err!("encountered unknown column"));
			}
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
				let value = lexical::parse::<f64, _>(&value)
					.map_err(|_| err!("unable to parse \"{}\" as a number", value))?;
				example.insert(
					key,
					serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()),
				);
			}
		}
	}
	let prediction = crate::common::predict::predict(model, example);
	Ok(prediction)
}
