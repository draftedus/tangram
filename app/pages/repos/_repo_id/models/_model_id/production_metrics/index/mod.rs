use crate::{
	common::{
		date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		production_metrics::get_production_metrics,
		time::format_date_window_interval,
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	production_metrics::ProductionPredictionMetricsOutput,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum Inner {
	Regressor(RegressorProductionMetricsOverview),
	Classifier(ClassifierProductionMetricsOverview),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorProductionMetricsOverview {
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	mse_chart: MSEChart,
	overall: RegressionProductionMetrics,
	true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TrueValuesCountChartEntry {
	label: String,
	count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MSEChart {
	data: Vec<MSEChartEntry>,
	training_mse: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MSEChartEntry {
	label: String,
	mse: Option<f32>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressionProductionMetrics {
	mse: TrainingProductionMetrics,
	rmse: TrainingProductionMetrics,
	true_values_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingProductionMetrics {
	production: Option<f32>,
	training: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassifierProductionMetricsOverview {
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	overall: ClassificationOverallProductionMetrics,
	id: String,
	accuracy_chart: AccuracyChart,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AccuracyChart {
	data: Vec<AccuracyChartEntry>,
	training_accuracy: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AccuracyChartEntry {
	accuracy: Option<f32>,
	label: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassificationOverallProductionMetrics {
	accuracy: TrainingProductionMetrics,
	class_metrics_table: Vec<ClassMetricsTableEntry>,
	true_values_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetricsTableEntry {
	class_name: String,
	precision: TrainingProductionMetrics,
	recall: TrainingProductionMetrics,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let (date_window, date_window_interval) = get_date_window_and_interval(&search_params)?;
	let props = props(
		request,
		context,
		model_id,
		date_window,
		date_window_interval,
	)
	.await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_metrics/",
		props,
	)?;
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
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
) -> Result<Props> {
	let timezone = get_timezone(&request);
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
	let production_metrics =
		get_production_metrics(&mut db, &model, date_window, date_window_interval, timezone)
			.await?;
	let inner = match &model {
		tangram_core::model::Model::Regressor(model) => {
			let training_metrics = &model.test_metrics;
			let true_values_count = production_metrics.overall.true_values_count;
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::Regression(metrics) => metrics,
						_ => unreachable!(),
					});
			let overall = RegressionProductionMetrics {
				mse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.mse),
					training: training_metrics.mse,
				},
				rmse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.rmse),
					training: training_metrics.rmse,
				},
				true_values_count,
			};
			let mse_chart = {
				let data = production_metrics
					.intervals
					.iter()
					.map(|interval| {
						let label = format_date_window_interval(
							interval.start_date,
							date_window_interval,
							timezone,
						);
						let mse = interval
							.prediction_metrics
							.as_ref()
							.map(|prediction_metrics| {
								if let ProductionPredictionMetricsOutput::Regression(
									predicion_metrics,
								) = prediction_metrics
								{
									predicion_metrics.mse
								} else {
									unreachable!()
								}
							});
						MSEChartEntry { label, mse }
					})
					.collect();
				MSEChart {
					data,
					training_mse: training_metrics.mse,
				}
			};
			let true_values_count_chart = production_metrics
				.intervals
				.iter()
				.map(|interval| TrueValuesCountChartEntry {
					count: interval.true_values_count,
					label: format_date_window_interval(
						interval.start_date,
						date_window_interval,
						timezone,
					),
				})
				.collect();
			Inner::Regressor(RegressorProductionMetricsOverview {
				date_window,
				date_window_interval,
				mse_chart,
				overall,
				true_values_count_chart,
			})
		}
		tangram_core::model::Model::Classifier(model) => {
			let training_metrics = &model.test_metrics;
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::Classification(metrics) => metrics,
						_ => unreachable!(),
					});
			let true_values_count_chart = production_metrics
				.intervals
				.iter()
				.map(|interval| TrueValuesCountChartEntry {
					count: interval.true_values_count,
					label: format_date_window_interval(
						interval.start_date,
						date_window_interval,
						timezone,
					),
				})
				.collect();
			let accuracy_chart = {
				let data = production_metrics
					.intervals
					.iter()
					.map(|interval| {
						let label = format_date_window_interval(
							interval.start_date,
							date_window_interval,
							timezone,
						);
						let accuracy =
							interval
								.prediction_metrics
								.as_ref()
								.map(|prediction_metrics| {
									if let ProductionPredictionMetricsOutput::Classification(
										predicion_metrics,
									) = prediction_metrics
									{
										predicion_metrics.accuracy
									} else {
										unreachable!()
									}
								});
						AccuracyChartEntry { label, accuracy }
					})
					.collect();
				AccuracyChart {
					data,
					training_accuracy: training_metrics.accuracy,
				}
			};
			let true_values_count = production_metrics.overall.true_values_count;
			let training_class_metrics = &training_metrics.class_metrics;
			let production_accuracy = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.accuracy);
			let production_class_metrics = overall_production_metrics
				.map(|production_metrics| production_metrics.class_metrics);
			let classes = model.classes();
			let class_metrics_table = training_class_metrics
				.iter()
				.zip(classes)
				.enumerate()
				.map(|(class_index, (training_class_metrics, class_name))| {
					let precision = production_class_metrics
						.as_ref()
						.map(|p| p[class_index].precision);
					let recall = production_class_metrics
						.as_ref()
						.map(|p| p[class_index].recall);
					ClassMetricsTableEntry {
						precision: TrainingProductionMetrics {
							training: training_class_metrics.precision,
							production: precision,
						},
						recall: TrainingProductionMetrics {
							training: training_class_metrics.recall,
							production: recall,
						},
						class_name: class_name.to_owned(),
					}
				})
				.collect();
			let overall = ClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: training_metrics.accuracy,
				},
				class_metrics_table,
				true_values_count,
			};
			Inner::Classifier(ClassifierProductionMetricsOverview {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: model_id.to_string(),
				accuracy_chart,
				overall,
			})
		}
	};
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}
