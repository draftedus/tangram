use super::props::{Pagination, PredictionTable, PredictionTableRow, Props};
use crate::{
	common::{
		error::Error,
		monitor_event::PredictOutput,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use chrono::prelude::*;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use std::collections::BTreeMap;
use tangram_util::error::Result;
use tangram_util::id::Id;

const N_PREDICTIONS_PER_PAGE: i64 = 10;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let after: Option<i64> = search_params
		.as_ref()
		.and_then(|s| s.get("after"))
		.and_then(|t| t.parse().ok());
	let before: Option<i64> = search_params
		.as_ref()
		.and_then(|s| s.get("before"))
		.and_then(|t| t.parse().ok());
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let rows = match (after, before) {
		(Some(after), None) => {
			let rows = sqlx::query(
				"
					select
						date,
						identifier,
						input,
						output
					from (
						select
							date,
							identifier,
							input,
							output
						from predictions
						where
								model_id = $1
						and date > $2
						order by date asc
						limit $3
					)
					order by date desc
				",
			)
			.bind(&model_id.to_string())
			.bind(after)
			.bind(N_PREDICTIONS_PER_PAGE)
			.fetch_all(&mut *db)
			.await?;
			rows
		}
		(None, Some(before)) => {
			sqlx::query(
				"
				select
					date,
					identifier,
					input,
					output
				from predictions
					where
						model_id = $1
				and date < $2
				order by date desc
				limit $3
			",
			)
			.bind(&model_id.to_string())
			.bind(before)
			.bind(N_PREDICTIONS_PER_PAGE)
			.fetch_all(&mut *db)
			.await?
		}
		(None, None) => {
			sqlx::query(
				"
					select
						date,
						identifier,
						input,
						output
					from predictions
						where
							model_id = $1
					order by date desc
					limit $2
				",
			)
			.bind(&model_id.to_string())
			.bind(N_PREDICTIONS_PER_PAGE)
			.fetch_all(&mut *db)
			.await?
		}
		_ => unreachable!(),
	};
	let first_row_timestamp = rows.first().map(|row| row.get::<i64, _>(0));
	let last_row_timestamp = rows.last().map(|row| row.get::<i64, _>(0));
	let (newer_predictions_exist, older_predictions_exist) =
		match (first_row_timestamp, last_row_timestamp) {
			(Some(first_row_timestamp), Some(last_row_timestamp)) => {
				let newer_predictions_exist: bool = sqlx::query(
					"
						select count(*) > 0
							from predictions
							where model_id = $1 and date > $2
					",
				)
				.bind(&model_id.to_string())
				.bind(first_row_timestamp)
				.fetch_one(&mut *db)
				.await?
				.get(0);
				let older_predictions_exist: bool = sqlx::query(
					"
					select count(*) > 0
						from predictions
						where model_id = $1 and date < $2
				",
				)
				.bind(&model_id.to_string())
				.bind(last_row_timestamp)
				.fetch_one(&mut *db)
				.await?
				.get(0);
				(newer_predictions_exist, older_predictions_exist)
			}
			(_, _) => (false, false),
		};
	let prediction_table_rows: Vec<PredictionTableRow> = rows
		.iter()
		.map(|row| {
			let date = row.get::<i64, _>(0);
			let date: DateTime<Utc> = Utc.timestamp(date, 0);
			let identifier: String = row.get(1);
			let output: String = row.get(3);
			let output: Vec<u8> = base64::decode(output).unwrap();
			let output: PredictOutput = serde_json::from_slice(&output).unwrap();
			let output = match output {
				PredictOutput::Regression(output) => output.value.to_string(),
				PredictOutput::BinaryClassification(output) => output.class_name,
				PredictOutput::MulticlassClassification(output) => output.class_name,
			};
			PredictionTableRow {
				date: date.to_rfc3339(),
				identifier,
				output,
			}
		})
		.collect();
	db.commit().await?;
	let pagination = Pagination {
		after: if newer_predictions_exist {
			first_row_timestamp.and_then(|t| t.to_usize())
		} else {
			None
		},
		before: if older_predictions_exist {
			last_row_timestamp.and_then(|t| t.to_usize())
		} else {
			None
		},
	};
	let props = Props {
		model_layout_info,
		prediction_table: if prediction_table_rows.is_empty() {
			None
		} else {
			Some(PredictionTable {
				rows: prediction_table_rows,
			})
		},
		pagination,
	};
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_predictions/",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
