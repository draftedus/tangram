use crate::{
	error::Error,
	monitor_event::{
		ClassificationOutput, MonitorEvent, NumberOrString, Output, PredictionMonitorEvent,
		RegressionOutput, TrueValueMonitorEvent,
	},
	production_metrics::ProductionMetrics,
	production_stats::ProductionStats,
	Context,
};
use anyhow::{format_err, Result};
use chrono::prelude::*;
use hyper::{body::to_bytes, Body, Request, Response, StatusCode};
use std::collections::BTreeMap as Map;
use std::sync::Arc;
use tangram_core::id::Id;
use tangram_core::{metrics::RunningMetric, types};
use tokio_postgres as postgres;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum MonitorEventSet {
	Single(MonitorEvent),
	Multiple(Vec<MonitorEvent>),
}

pub async fn track(mut request: Request<Body>, context: Arc<Context>) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let monitor_events: MonitorEventSet =
		serde_json::from_slice(&data).map_err(|_| Error::BadRequest)?;
	let monitor_events = match monitor_events {
		MonitorEventSet::Single(s) => vec![s],
		MonitorEventSet::Multiple(m) => m,
	};
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let mut models = Map::new();
	for monitor_event in monitor_events {
		match monitor_event {
			MonitorEvent::Prediction(monitor_event) => {
				let handle_prediction_result =
					handle_prediction_monitor_event(&db, &mut models, monitor_event).await;
				if handle_prediction_result.is_err() {
					return Err(Error::BadRequest.into());
				}
			}
			MonitorEvent::TrueValue(monitor_event) => {
				let handle_true_value_result =
					handle_true_value_monitor_event(&db, &mut models, monitor_event).await;
				if handle_true_value_result.is_err() {
					return Err(Error::BadRequest.into());
				}
			}
		}
	}
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::ACCEPTED)
		.body(Body::empty())?)
}

async fn handle_prediction_monitor_event(
	tx: &postgres::Transaction<'_>,
	models: &mut Map<Id, types::Model>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let model = match models.get(&model_id) {
		Some(m) => m,
		None => {
			let model = crate::model::get_model(&tx, model_id).await?;
			models.insert(model_id, model);
			models.get(&model_id).unwrap()
		}
	};
	write_prediction_monitor_event(&tx, model_id, &monitor_event).await?;
	insert_or_update_production_stats_for_monitor_event(&tx, model_id, &model, monitor_event)
		.await?;
	Ok(())
}

async fn handle_true_value_monitor_event(
	tx: &postgres::Transaction<'_>,
	models: &mut Map<Id, types::Model>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let model = match models.get(&model_id) {
		Some(m) => m,
		None => {
			let model = crate::model::get_model(&tx, monitor_event.model_id).await?;
			models.insert(model_id, model);
			models.get(&model_id).unwrap()
		}
	};
	write_true_value_monitor_event(&tx, model_id, &monitor_event).await?;
	insert_or_update_production_metrics_for_monitor_event(&tx, model_id, &model, monitor_event)
		.await?;
	Ok(())
}

async fn write_prediction_monitor_event(
	tx: &postgres::Transaction<'_>,
	model_id: Id,
	monitor_event: &PredictionMonitorEvent,
) -> Result<()> {
	let prediction_monitor_event_id = Id::new();
	let created_at: DateTime<Utc> = Utc::now();
	let identifier = monitor_event.identifier.as_string();
	let date = &monitor_event.date;
	let input = serde_json::to_vec(&monitor_event.input)?;
	let output = serde_json::to_vec(&monitor_event.output)?;
	tx.execute(
		"
			insert into predictions
				(id, model_id, date, created_at, identifier, input, output)
			values
				($1, $2, $3, $4, $5, $6, $7)
		",
		&[
			&prediction_monitor_event_id,
			&model_id,
			&date,
			&created_at,
			&identifier,
			&input,
			&output,
		],
	)
	.await?;
	Ok(())
}

async fn write_true_value_monitor_event(
	tx: &postgres::Transaction<'_>,
	model_id: Id,
	monitor_event: &TrueValueMonitorEvent,
) -> Result<()> {
	let true_value_monitor_event_id = Id::new();
	let created_at: DateTime<Utc> = Utc::now();
	let date = monitor_event.date;
	let identifier = monitor_event.identifier.as_string();
	tx.execute(
		"
			insert into true_values
				(id, model_id, date, created_at, identifier, value)
			values
				($1, $2, $3, $4, $5, $6)
		",
		&[
			&true_value_monitor_event_id,
			&model_id,
			&date,
			&created_at,
			&identifier,
			&monitor_event.true_value.as_string(),
		],
	)
	.await?;
	Ok(())
}

async fn insert_or_update_production_stats_for_monitor_event(
	tx: &postgres::Transaction<'_>,
	model_id: Id,
	model: &types::Model,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let date = monitor_event.date;
	let hour = Utc
		.ymd(date.year(), date.month(), date.day())
		.and_hms(date.hour(), 0, 0);
	let rows = tx
		.query(
			"
				select
					data
				from production_stats
				where
					model_id = $1
				and
				  hour = $2
			",
			&[&model_id, &hour],
		)
		.await?;
	if let Some(row) = rows.get(0) {
		let data: Vec<u8> = row.get(0);
		let mut production_stats: ProductionStats = serde_json::from_slice(&data)?;
		production_stats.update(monitor_event);
		let data = serde_json::to_vec(&production_stats)?;
		tx.execute(
			"
				update
					production_stats
				set
					data = $1
				where
					model_id = $2
				and
					hour = $3
			",
			&[&data, &model_id, &hour],
		)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_stats = ProductionStats::new(&model, start_date, end_date);
		production_stats.update(monitor_event);
		let data = serde_json::to_vec(&production_stats)?;
		tx.execute(
			"
				insert into production_stats
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
			&[&model_id, &data, &hour],
		)
		.await?;
	}
	Ok(())
}
async fn insert_or_update_production_metrics_for_monitor_event(
	tx: &postgres::Transaction<'_>,
	model_id: Id,
	model: &types::Model,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string();
	let hour = monitor_event
		.date
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.with_nanosecond(0)
		.unwrap();
	let rows = tx
		.query(
			"
				select
					predictions.output
				from
					predictions
				where
					predictions.model_id = $1
				and
					predictions.identifier = $2
			",
			&[&model_id, &identifier],
		)
		.await?;
	let row = rows
		.get(0)
		.ok_or_else(|| format_err!("no prediction with identifier {}", identifier))?;
	let output: Vec<u8> = row.get(0);
	let output: Output = serde_json::from_slice(output.as_slice())?;
	let prediction = match output {
		Output::Regression(RegressionOutput { value }) => NumberOrString::Number(value),
		Output::Classification(ClassificationOutput { class_name, .. }) => {
			NumberOrString::String(class_name)
		}
	};
	let rows = tx
		.query(
			"
				select
					data
				from production_metrics
				where
					model_id = $1
				and
					hour = $2
			",
			&[&model_id, &hour],
		)
		.await?;
	if let Some(row) = rows.get(0) {
		let data: Vec<u8> = row.get(0);
		let mut production_metrics: ProductionMetrics = serde_json::from_slice(&data)?;
		production_metrics.update((prediction, monitor_event.true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		tx.execute(
			"
					update
						production_metrics
					set
						data = $1
					where
						model_id = $2
					and
						hour = $3
				",
			&[&data, &model_id, &hour],
		)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_metrics = ProductionMetrics::new(&model, start_date, end_date);
		production_metrics.update((prediction, monitor_event.true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		tx.execute(
			"
				insert into production_metrics
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
			&[&model_id, &data, &hour],
		)
		.await?;
	}
	Ok(())
}
