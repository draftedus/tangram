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
use sqlx::prelude::*;
use std::{collections::BTreeMap, sync::Arc};
use tangram_core::metrics::Metric;
use tangram_id::Id;

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
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let mut models = BTreeMap::new();
	for monitor_event in monitor_events {
		match monitor_event {
			MonitorEvent::Prediction(monitor_event) => {
				let handle_prediction_result =
					handle_prediction_monitor_event(&mut db, &mut models, monitor_event).await;
				if handle_prediction_result.is_err() {
					println!("{:?}", handle_prediction_result);
					return Err(Error::BadRequest.into());
				}
			}
			MonitorEvent::TrueValue(monitor_event) => {
				let handle_true_value_result =
					handle_true_value_monitor_event(&mut db, &mut models, monitor_event).await;
				if handle_true_value_result.is_err() {
					println!("{:?}", handle_true_value_result);
					return Err(Error::BadRequest.into());
				}
			}
		}
	}
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::ACCEPTED)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn handle_prediction_monitor_event(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	models: &mut BTreeMap<Id, tangram_core::model::Model>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let model = match models.get(&model_id) {
		Some(m) => m,
		None => {
			let model = get_model(&mut db, model_id).await?;
			models.insert(model_id, model);
			models.get(&model_id).unwrap()
		}
	};
	write_prediction_monitor_event(&mut db, model_id, &monitor_event).await?;
	insert_or_update_production_stats_for_monitor_event(&mut db, model_id, &model, monitor_event)
		.await?;
	Ok(())
}

async fn handle_true_value_monitor_event(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	models: &mut BTreeMap<Id, tangram_core::model::Model>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let model = match models.get(&model_id) {
		Some(m) => m,
		None => {
			let model = get_model(&mut db, monitor_event.model_id).await?;
			models.insert(model_id, model);
			models.get(&model_id).unwrap()
		}
	};
	write_true_value_monitor_event(&mut db, model_id, &monitor_event).await?;
	insert_or_update_production_metrics_for_monitor_event(&mut db, model_id, &model, monitor_event)
		.await?;
	Ok(())
}

async fn write_prediction_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &PredictionMonitorEvent,
) -> Result<()> {
	let prediction_monitor_event_id = Id::new();
	let now = Utc::now().timestamp();
	let identifier = monitor_event.identifier.as_string().to_string();
	let date = &monitor_event.date;
	let input = serde_json::to_vec(&monitor_event.input)?;
	let output = serde_json::to_vec(&monitor_event.output)?;
	sqlx::query(
		"
			insert into predictions
				(id, model_id, date, created_at, identifier, input, output)
			values
				(?1, ?2, ?3, ?4, ?5, ?6, ?7)
		",
	)
	.bind(&prediction_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&now)
	.bind(&identifier)
	.bind(&base64::encode(input))
	.bind(&base64::encode(output))
	.execute(&mut *db)
	.await?;
	Ok(())
}

async fn write_true_value_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &TrueValueMonitorEvent,
) -> Result<()> {
	let true_value_monitor_event_id = Id::new();
	let now = Utc::now().timestamp();
	let date = monitor_event.date;
	let identifier = monitor_event.identifier.as_string().to_string();
	let true_value = &monitor_event.true_value.as_string().to_string();
	sqlx::query(
		"
			insert into true_values
				(id, model_id, date, created_at, identifier, value)
			values
				(?1, ?2, ?3, ?4, ?5, ?6)
		",
	)
	.bind(&true_value_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&now)
	.bind(&identifier)
	.bind(&true_value)
	.execute(&mut *db)
	.await?;
	Ok(())
}

async fn insert_or_update_production_stats_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: &tangram_core::model::Model,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let date = monitor_event.date;
	let hour = Utc
		.ymd(date.year(), date.month(), date.day())
		.and_hms(date.hour(), 0, 0);
	let rows = sqlx::query(
		"
			select
				data
			from production_stats
			where
				model_id = ?1
			and
				hour = ?2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(&mut *db)
	.await?;
	if let Some(row) = rows.get(0) {
		let data: String = row.get(0);
		let data: Vec<u8> = base64::decode(data)?;
		let mut production_stats: ProductionStats = serde_json::from_slice(&data)?;
		production_stats.update(monitor_event);
		let data = serde_json::to_vec(&production_stats)?;
		sqlx::query(
			"
				update
					production_stats
				set
					data = ?1
				where
					model_id = ?2
				and
					hour = ?3
			",
		)
		.bind(&base64::encode(data))
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_stats = ProductionStats::new(&model, start_date, end_date);
		production_stats.update(monitor_event);
		let data = serde_json::to_vec(&production_stats)?;
		sqlx::query(
			"
				insert into production_stats
					(model_id, data, hour)
				values
					(?1, ?2, ?3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&base64::encode(data))
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}

async fn insert_or_update_production_metrics_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: &tangram_core::model::Model,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string().to_string();
	let hour = monitor_event
		.date
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.with_nanosecond(0)
		.unwrap();
	let rows = sqlx::query(
		"
			select
				predictions.output
			from
				predictions
			where
				predictions.model_id = ?1
			and
				predictions.identifier = ?2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_all(&mut *db)
	.await?;
	let row = rows
		.get(0)
		.ok_or_else(|| format_err!("no prediction with identifier {}", identifier))?;
	let output: String = row.get(0);
	let output: Vec<u8> = base64::decode(output)?;
	let output: Output = serde_json::from_slice(output.as_slice())?;
	let prediction = match output {
		Output::Regression(RegressionOutput { value }) => NumberOrString::Number(value),
		Output::Classification(ClassificationOutput { class_name, .. }) => {
			NumberOrString::String(class_name)
		}
	};
	let rows = sqlx::query(
		"
			select
				data
			from production_metrics
			where
				model_id = ?1
			and
				hour = ?2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(&mut *db)
	.await?;
	if let Some(row) = rows.get(0) {
		let data: String = row.get(0);
		let data: Vec<u8> = base64::decode(data)?;
		let mut production_metrics: ProductionMetrics = serde_json::from_slice(&data)?;
		production_metrics.update((prediction, monitor_event.true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		sqlx::query(
			"
				update
					production_metrics
				set
					data = ?1
				where
					model_id = ?2
				and
					hour = ?3
			",
		)
		.bind(&base64::encode(data))
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_metrics = ProductionMetrics::new(&model, start_date, end_date);
		production_metrics.update((prediction, monitor_event.true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		sqlx::query(
			"
				insert into production_metrics
					(model_id, data, hour)
				values
					(?1, ?2, ?3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&base64::encode(data))
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}

/// Retrieves the model with the specified id. Errors if the model is not found.
pub async fn get_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<tangram_core::model::Model> {
	let data: String = sqlx::query(
		"
			select
				data
			from models
			where
				models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?
	.get(0);
	let data: Vec<u8> = base64::decode(data)?;
	let model = tangram_core::model::Model::from_slice(&data.as_slice())?;
	Ok(model)
}
