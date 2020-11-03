use super::props::{
	AccuracyChart, AccuracyChartEntry, BinaryClassificationOverallProductionMetrics,
	BinaryClassifierProductionMetricsOverview, ClassMetricsTableEntry, Inner, MSEChart,
	MSEChartEntry, MulticlassClassificationOverallProductionMetrics,
	MulticlassClassifierProductionMetricsOverview, Props, RegressionProductionMetrics,
	RegressorProductionMetricsOverview, TrainingProductionMetrics, TrueValuesCountChartEntry,
};
use crate::{
	common::{
		date_window::get_date_window_and_interval,
		error::Error,
		model::get_model,
		production_metrics::get_production_metrics,
		time::format_date_window_interval,
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	production_metrics::ProductionPredictionMetricsOutput,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_util::{error::Result, id::Id, zip};

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let (date_window, date_window_interval) = get_date_window_and_interval(&search_params)?;
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
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
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
		tangram_core::model::Model::BinaryClassifier(model) => {
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::BinaryClassification(metrics) => metrics,
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
									if let ProductionPredictionMetricsOutput::BinaryClassification(
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
				let default_threshold_test_metrics = model
					.test_metrics
					.thresholds
					.get(model.test_metrics.thresholds.len() / 2)
					.unwrap();
				let training_accuracy = default_threshold_test_metrics.accuracy;
				AccuracyChart {
					data,
					training_accuracy,
				}
			};
			let true_values_count = production_metrics.overall.true_values_count;
			let production_accuracy = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.accuracy);
			let production_precision = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.precision);
			let production_recall = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.recall);
			let default_threshold_test_metrics = model
				.test_metrics
				.thresholds
				.get(model.test_metrics.thresholds.len() / 2)
				.unwrap();
			let overall = BinaryClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: default_threshold_test_metrics.accuracy,
				},
				precision: TrainingProductionMetrics {
					production: production_precision,
					training: default_threshold_test_metrics.precision,
				},
				recall: TrainingProductionMetrics {
					production: production_recall,
					training: default_threshold_test_metrics.recall,
				},
				true_values_count,
			};
			Inner::BinaryClassifier(BinaryClassifierProductionMetricsOverview {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: model_id.to_string(),
				accuracy_chart,
				overall,
			})
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			let training_metrics = &model.test_metrics;
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::MulticlassClassification(metrics) => {
							metrics
						}
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
									if let ProductionPredictionMetricsOutput::MulticlassClassification(
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
			let class_metrics_table = zip!(training_class_metrics, model.classes.iter())
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
			let overall = MulticlassClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: training_metrics.accuracy,
				},
				class_metrics_table,
				true_values_count,
			};
			Inner::MulticlassClassifier(MulticlassClassifierProductionMetricsOverview {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: model_id.to_string(),
				accuracy_chart,
				overall,
			})
		}
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	let props = Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	};
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
