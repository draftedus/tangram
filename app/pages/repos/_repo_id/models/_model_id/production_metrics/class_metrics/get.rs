use super::props::{
	ClassMetricsEntry, Comparison, ConfusionMatrix, IntervalEntry, OverallClassMetrics,
	OverallClassMetricsEntry, Props, TrainingProductionMetrics,
};
use crate::Context;
use crate::{
	common::{
		date_window::get_date_window_and_interval,
		error::Error,
		model::get_model,
		production_metrics::{get_production_metrics, GetProductionMetricsOutput},
		time::format_date_window_interval,
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	production_metrics::ProductionPredictionMetricsOutput,
};
use hyper::{Body, Request, Response, StatusCode};
use itertools::izip;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use tangram_util::error::Result;
use tangram_util::id::Id;

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
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let production_metrics =
		get_production_metrics(&mut db, &model, date_window, date_window_interval, timezone)
			.await?;
	let model = match model {
		tangram_core::model::Model::MulticlassClassifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
	};
	let classes = model.classes;
	let GetProductionMetricsOutput {
		overall, intervals, ..
	} = production_metrics;
	let overall_prediction_metrics =
		overall
			.prediction_metrics
			.map(|prediction_metrics| match prediction_metrics {
				ProductionPredictionMetricsOutput::Regression(_) => unreachable!(),
				ProductionPredictionMetricsOutput::BinaryClassification(_) => unreachable!(),
				ProductionPredictionMetricsOutput::MulticlassClassification(prediction_metrics) => {
					prediction_metrics
				}
			});
	let training_class_metrics = &model.test_metrics.class_metrics;
	let overall_class_metrics: Vec<OverallClassMetricsEntry> =
		izip!(training_class_metrics, classes.iter())
			.enumerate()
			.map(|(class_index, (training_class_metrics, class_name))| {
				let production_class_metrics = overall_prediction_metrics
					.as_ref()
					.map(|prediction_metrics| &prediction_metrics.class_metrics[class_index]);
				let training_tn = training_class_metrics.true_negatives;
				let training_fp = training_class_metrics.false_positives;
				let training_tp = training_class_metrics.true_positives;
				let training_fn = training_class_metrics.false_negatives;
				let training_total = training_tn + training_fp + training_tp + training_fn;
				let training_tn_fraction =
					training_tn.to_f32().unwrap() / training_total.to_f32().unwrap();
				let training_fp_fraction =
					training_fp.to_f32().unwrap() / training_total.to_f32().unwrap();
				let training_tp_fraction =
					training_tp.to_f32().unwrap() / training_total.to_f32().unwrap();
				let training_fn_fraction =
					training_fn.to_f32().unwrap() / training_total.to_f32().unwrap();
				let production_tp = production_class_metrics.map(|m| m.true_positives);
				let production_fp = production_class_metrics.map(|m| m.false_positives);
				let production_tn = production_class_metrics.map(|m| m.true_negatives);
				let production_fn = production_class_metrics.map(|m| m.false_negatives);
				let production_total = production_class_metrics.map(|m| {
					m.false_positives + m.false_negatives + m.true_positives + m.true_negatives
				});
				let production_tp_fraction = production_tp
					.map(|p| p.to_f32().unwrap() / production_total.unwrap().to_f32().unwrap());
				let production_fp_fraction = production_fp
					.map(|p| p.to_f32().unwrap() / production_total.unwrap().to_f32().unwrap());
				let production_tn_fraction = production_tn
					.map(|p| p.to_f32().unwrap() / production_total.unwrap().to_f32().unwrap());
				let production_fn_fraction = production_fn
					.map(|p| p.to_f32().unwrap() / production_total.unwrap().to_f32().unwrap());
				let confusion_matrix = ConfusionMatrix {
					false_negatives: production_fn,
					true_negatives: production_tn,
					true_positives: production_tp,
					false_positives: production_fp,
				};
				let comparison = Comparison {
					false_negative_fraction: TrainingProductionMetrics {
						production: production_fn_fraction,
						training: training_fn_fraction,
					},
					false_positive_fraction: TrainingProductionMetrics {
						production: production_fp_fraction,
						training: training_fp_fraction,
					},
					true_positive_fraction: TrainingProductionMetrics {
						production: production_tp_fraction,
						training: training_tp_fraction,
					},
					true_negative_fraction: TrainingProductionMetrics {
						production: production_tn_fraction,
						training: training_tn_fraction,
					},
				};
				OverallClassMetricsEntry {
					class_name: class_name.clone(),
					comparison,
					confusion_matrix,
					f1_score: TrainingProductionMetrics {
						training: training_class_metrics.f1_score,
						production: production_class_metrics.map(|m| m.f1_score),
					},
					precision: TrainingProductionMetrics {
						production: production_class_metrics.map(|m| m.precision),
						training: training_class_metrics.f1_score,
					},
					recall: TrainingProductionMetrics {
						training: training_class_metrics.recall,
						production: production_class_metrics.map(|m| m.recall),
					},
				}
			})
			.collect();
	let overall = OverallClassMetrics {
		label: format_date_window_interval(overall.start_date, date_window_interval, timezone),
		class_metrics: overall_class_metrics,
	};
	let class_metrics: Vec<ClassMetricsEntry> = classes
		.iter()
		.enumerate()
		.map(|(class_index, class_name)| {
			let intervals = intervals
				.iter()
				.map(|interval| {
					let metrics =
						interval
							.prediction_metrics
							.as_ref()
							.map(|metrics| match metrics {
								ProductionPredictionMetricsOutput::Regression(_) => unreachable!(),
								ProductionPredictionMetricsOutput::BinaryClassification(_) => {
									unreachable!()
								}
								ProductionPredictionMetricsOutput::MulticlassClassification(
									prediction_metrics,
								) => prediction_metrics,
							});
					let production_f1_score =
						metrics.map(|m| m.class_metrics[class_index].f1_score);
					let production_recall = metrics.map(|m| m.class_metrics[class_index].recall);
					let production_precision =
						metrics.map(|m| m.class_metrics[class_index].precision);
					IntervalEntry {
						label: format_date_window_interval(
							interval.start_date,
							date_window_interval,
							timezone,
						),
						f1_score: TrainingProductionMetrics {
							production: production_f1_score,
							training: training_class_metrics[class_index].f1_score,
						},
						precision: TrainingProductionMetrics {
							production: production_precision,
							training: training_class_metrics[class_index].precision,
						},
						recall: TrainingProductionMetrics {
							production: production_recall,
							training: training_class_metrics[class_index].precision,
						},
					}
				})
				.collect();
			ClassMetricsEntry {
				class_name: class_name.clone(),
				intervals,
			}
		})
		.collect();
	db.commit().await?;
	let class = search_params.and_then(|s| s.get("class").map(|class| class.to_owned()));
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		1
	};
	let class = class.unwrap_or_else(|| classes.get(class_index).unwrap().to_owned());
	let props = Props {
		id: model_id.to_string(),
		class_metrics,
		date_window,
		date_window_interval,
		classes: classes.to_owned(),
		overall,
		model_layout_info,
		class,
	};
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_metrics/class_metrics",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
