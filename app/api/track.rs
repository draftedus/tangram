use std::collections::BTreeMap;
use tangram_app_common::{
	error::{bad_request, service_unavailable},
	model::get_model,
	monitor_event::{
		BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
		NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
		TrueValueMonitorEvent,
	},
	production_metrics::ProductionMetrics,
	production_stats::ProductionStats,
	Context,
};
use tangram_deps::{
	base64, chrono, chrono::prelude::*, http, hyper, num_traits::ToPrimitive, serde_json, sqlx,
	sqlx::prelude::*,
};
use tangram_metrics::StreamingMetric;
use tangram_util::{err, error::Result, id::Id};

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum MonitorEventSet {
	Single(MonitorEvent),
	Multiple(Vec<MonitorEvent>),
}

pub async fn post(
	context: &Context,
	mut request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(bytes) => bytes,
		Err(_) => return Ok(bad_request()),
	};
	let monitor_events: MonitorEventSet = match serde_json::from_slice(&data) {
		Ok(monitor_events) => monitor_events,
		Err(_) => return Ok(bad_request()),
	};
	let monitor_events = match monitor_events {
		MonitorEventSet::Single(monitor_event) => vec![monitor_event],
		MonitorEventSet::Multiple(monitor_event) => monitor_event,
	};
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let mut models = BTreeMap::new();
	for monitor_event in monitor_events {
		match monitor_event {
			MonitorEvent::Prediction(monitor_event) => {
				let handle_prediction_result =
					handle_prediction_monitor_event(&mut db, &mut models, monitor_event).await;
				if handle_prediction_result.is_err() {
					return Ok(bad_request());
				}
			}
			MonitorEvent::TrueValue(monitor_event) => {
				let handle_true_value_result =
					handle_true_value_monitor_event(&mut db, &mut models, monitor_event).await;
				if handle_true_value_result.is_err() {
					return Ok(bad_request());
				}
			}
		}
	}
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::ACCEPTED)
		.body(hyper::Body::empty())
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
		Some(model) => model,
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
		Some(model) => model,
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
	let identifier = monitor_event.identifier.as_string();
	let date = &monitor_event.date;
	let input = serde_json::to_vec(&monitor_event.input)?;
	let output = serde_json::to_vec(&monitor_event.output)?;
	sqlx::query(
		"
			insert into predictions
				(id, model_id, date, identifier, input, output)
			values
				($1, $2, $3, $4, $5, $6)
		",
	)
	.bind(&prediction_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
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
	let date = monitor_event.date;
	let identifier = monitor_event.identifier.as_string();
	let true_value = &monitor_event.true_value.to_string();
	sqlx::query(
		"
			insert into true_values
				(id, model_id, date, identifier, value)
			values
				($1, $2, $3, $4, $5)
		",
	)
	.bind(&true_value_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
	.bind(&true_value.to_string())
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
				model_id = $1
			and
				hour = $2
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
					data = $1
				where
					model_id = $2
				and
					hour = $3
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
					($1, $2, $3)
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
				predictions.model_id = $1
			and
				predictions.identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_all(&mut *db)
	.await?;
	let row = rows
		.get(0)
		.ok_or_else(|| err!("no prediction with identifier {}", identifier))?;
	let output: String = row.get(0);
	let output: Vec<u8> = base64::decode(output)?;
	let output: PredictOutput = serde_json::from_slice(output.as_slice())?;
	let prediction = match output {
		PredictOutput::Regression(RegressionPredictOutput { value }) => {
			NumberOrString::Number(value)
		}
		PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
		PredictOutput::MulticlassClassification(MulticlassClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
	};
	let true_value = match &monitor_event.true_value {
		serde_json::Value::Number(value) => {
			NumberOrString::Number(value.as_f64().unwrap().to_f32().unwrap())
		}
		serde_json::Value::String(value) => NumberOrString::String(value.clone()),
		_ => unimplemented!(),
	};
	let rows = sqlx::query(
		"
			select
				data
			from production_metrics
			where
				model_id = $1
			and
				hour = $2
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
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		sqlx::query(
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
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_vec(&production_metrics)?;
		sqlx::query(
			"
				insert into production_metrics
					(model_id, data, hour)
				values
					($1, $2, $3)
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
