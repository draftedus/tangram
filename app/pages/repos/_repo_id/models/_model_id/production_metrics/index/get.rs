use super::props::{
	AccuracyChart, AccuracyChartEntry, BinaryClassificationOverallProductionMetrics,
	BinaryClassifierProductionMetricsOverview, ClassMetricsTableEntry, Inner, MSEChart,
	MSEChartEntry, MulticlassClassificationOverallProductionMetrics,
	MulticlassClassifierProductionMetricsOverview, Props, RegressionProductionMetrics,
	RegressorProductionMetricsOverview, TrainingProductionMetrics, TrueValuesCountChartEntry,
};
use std::collections::BTreeMap;
use tangram_app_common::{
	date_window::get_date_window_and_interval,
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	production_metrics::get_production_metrics,
	production_metrics::ProductionPredictionMetricsOutput,
	time::format_date_window_interval,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{http, hyper, pinwheel::Pinwheel};
use tangram_util::{error::Result, id::Id, zip};

pub async fn get(
	pinwheel: &Pinwheel,
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<http::Response<hyper::Body>> {
	let (date_window, date_window_interval) = match get_date_window_and_interval(&search_params) {
		Some((date_window, date_window_interval)) => (date_window, date_window_interval),
		None => return Ok(bad_request()),
	};
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
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
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
						class_name: class_name.clone(),
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
	let html = pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_metrics/",
		props,
	)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
