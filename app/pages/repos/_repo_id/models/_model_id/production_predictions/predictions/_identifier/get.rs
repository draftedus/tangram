use super::props::{Found, Inner, Props};
use std::collections::BTreeMap;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	predict::{ColumnType, InputTable, InputTableRow, Prediction},
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{
	base64, chrono::prelude::*, chrono_tz::Tz, http, hyper, lexical, pinwheel::Pinwheel,
	serde_json, sqlx, sqlx::prelude::*,
};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	pinwheel: &Pinwheel,
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	identifier: &str,
) -> Result<http::Response<hyper::Body>> {
	let timezone = get_timezone(&request);
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
	let model = get_model(&mut db, model_id).await?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let row = sqlx::query(
		"
		select
			date,
			identifier,
			input,
			output
		from predictions
			where
				model_id = $1
			and identifier = $2
		order by date
		limit 10
		",
	)
	.bind(&model_id.to_string())
	.bind(identifier)
	.fetch_optional(&mut *db)
	.await?;
	let inner = match row {
		Some(row) => {
			let date: i64 = row.get(0);
			let date: DateTime<Tz> = Utc.timestamp(date, 0).with_timezone(&timezone);
			let input: String = row.get(2);
			let input: Vec<u8> = base64::decode(input).unwrap();
			let input: serde_json::Map<String, serde_json::Value> =
				serde_json::from_slice(&input).unwrap();
			let prediction_output = predict(model, input);
			db.commit().await?;
			Inner::Found(Found {
				input_table: prediction_output.input_table,
				prediction: prediction_output.prediction,
				date: date.to_string(),
			})
		}
		None => Inner::NotFound,
	};
	let props = Props {
		model_layout_info,
		identifier: identifier.to_owned(),
		inner,
	};
	let html = pinwheel.render_with_props(
		"/repos/_repo_id/models/_model_id/production_predictions/predictions/_identifier",
		props,
	)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

struct PredictionOutput {
	prediction: Prediction,
	input_table: InputTable,
}

fn predict(
	model: tangram_core::model::Model,
	input: serde_json::Map<String, serde_json::Value>,
) -> PredictionOutput {
	let column_stats = match &model {
		tangram_core::model::Model::Regressor(model) => &model.overall_column_stats,
		tangram_core::model::Model::BinaryClassifier(model) => &model.overall_column_stats,
		tangram_core::model::Model::MulticlassClassifier(model) => &model.overall_column_stats,
	};
	let mut column_lookup = BTreeMap::new();
	for column in column_stats.iter() {
		match column {
			tangram_core::model::ColumnStats::Number(number_column) => {
				column_lookup.insert(number_column.column_name.clone(), column);
			}
			tangram_core::model::ColumnStats::Enum(enum_column) => {
				column_lookup.insert(enum_column.column_name.clone(), column);
			}
			tangram_core::model::ColumnStats::Text(text_column) => {
				column_lookup.insert(text_column.column_name.clone(), column);
			}
			_ => unreachable!(),
		}
	}
	let mut example: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
	let mut input_table_rows = Vec::new();
	for (column_name, value) in input.into_iter() {
		if let Some(column) = column_lookup.get(&column_name) {
			match column {
				tangram_core::model::ColumnStats::Text(_) => {
					input_table_rows.push(InputTableRow {
						column_name: column_name.clone(),
						value: value.clone(),
						column_type: ColumnType::Text,
					});
					if let serde_json::Value::String(_) = value {
						example.insert(column_name, value);
					}
				}
				tangram_core::model::ColumnStats::Enum(_) => {
					input_table_rows.push(InputTableRow {
						column_name: column_name.clone(),
						value: value.clone(),
						column_type: ColumnType::Enum,
					});
					if let serde_json::Value::String(_) = value {
						example.insert(column_name, value);
					}
				}
				tangram_core::model::ColumnStats::Number(_) => {
					input_table_rows.push(InputTableRow {
						column_name: column_name.clone(),
						value: value.clone(),
						column_type: ColumnType::Number,
					});
					match value {
						serde_json::Value::Number(_) => {
							example.insert(column_name, value);
						}
						serde_json::Value::String(value) => {
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
								column_name,
								serde_json::Value::Number(
									serde_json::Number::from_f64(value).unwrap(),
								),
							);
						}
						_ => continue,
					}
				}
				tangram_core::model::ColumnStats::Unknown(_) => {
					input_table_rows.push(InputTableRow {
						column_name: column_name.clone(),
						value: value.clone(),
						column_type: ColumnType::Unknown,
					});
				}
			};
		} else {
			input_table_rows.push(InputTableRow {
				column_name,
				value: value.clone(),
				column_type: ColumnType::Unknown,
			})
		}
	}
	let prediction = tangram_app_common::predict::predict(model, example);
	PredictionOutput {
		input_table: InputTable {
			rows: input_table_rows,
		},
		prediction,
	}
}
