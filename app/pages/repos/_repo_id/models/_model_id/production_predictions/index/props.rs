use crate::{
	common::{
		error::Error,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		monitor_event::PredictOutput,
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{Body, Request};
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use tangram_util::id::Id;

const N_PREDICTIONS_PER_PAGE: i64 = 10;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	model_layout_info: ModelLayoutInfo,
	prediction_table: PredictionTable,
	pagination: Pagination,
}

#[derive(serde::Serialize, Debug)]
struct PredictionTable {
	rows: Vec<PredictionTableRow>,
}

#[derive(serde::Serialize, Debug)]
struct PredictionTableRow {
	date: String,
	identifier: String,
	output: String,
}

#[derive(serde::Serialize, Debug)]
struct Pagination {
	after: Option<usize>,
	before: Option<usize>,
}

#[derive(serde::Serialize, Debug)]
struct PaginationRange {
	start: usize,
	end: usize,
	total: usize,
}

pub async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	before: Option<i64>,
	after: Option<i64>,
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
								model_id = ?1
						and date > ?2
						order by date asc
						limit ?3
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
						model_id = ?1
				and date < ?2
				order by date desc
				limit ?3
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
							model_id = ?1
					order by date desc
					limit ?2
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
						select case when exists (
							select 1
							from predictions
							where model_id = ?1 and date > ?2
						)
						then 1 else 0 end
					",
				)
				.bind(&model_id.to_string())
				.bind(first_row_timestamp)
				.fetch_one(&mut *db)
				.await?
				.get(0);
				let older_predictions_exist: bool = sqlx::query(
					"
					select case when exists (
						select 1
						from predictions
						where model_id = ?1 and date < ?2
					)
					then 1 else 0 end
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
	Ok(Props {
		model_layout_info,
		prediction_table: PredictionTable {
			rows: prediction_table_rows,
		},
		pagination,
	})
}
