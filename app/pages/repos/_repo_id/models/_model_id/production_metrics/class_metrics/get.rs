use super::page::{
	render, ClassMetricsEntry, Comparison, ConfusionMatrix, IntervalEntry, OverallClassMetrics,
	OverallClassMetricsEntry, Props, TrainingProductionMetrics,
};
use std::collections::BTreeMap;
use tangram_app_common::{
	date_window::get_date_window_and_interval,
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	production_metrics::ProductionPredictionMetricsOutput,
	production_metrics::{get_production_metrics, GetProductionMetricsOutput},
	time::format_date_window_interval,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::{
	http, hyper,
	num_traits::ToPrimitive,
	pinwheel::{self, client},
};
use tangram_util::{error::Result, id::Id, zip};

pub async fn get(
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
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let production_metrics = get_production_metrics(
		&mut db,
		&model,
		date_window.clone(),
		date_window_interval.clone(),
		timezone,
	)
	.await?;
	let model = match model {
		tangram_core::model::Model::MulticlassClassifier(model) => model,
		_ => return Ok(bad_request()),
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
		zip!(training_class_metrics, classes.iter())
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
				let confusion_matrix =
					production_class_metrics.map(|production_class_metrics| ConfusionMatrix {
						false_negatives: production_class_metrics
							.false_negatives
							.to_usize()
							.unwrap(),
						true_negatives: production_class_metrics.true_negatives.to_usize().unwrap(),
						true_positives: production_class_metrics.true_positives.to_usize().unwrap(),
						false_positives: production_class_metrics
							.false_negatives
							.to_usize()
							.unwrap(),
					});
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
						training: training_class_metrics.precision,
					},
					recall: TrainingProductionMetrics {
						training: training_class_metrics.recall,
						production: production_class_metrics.map(|m| m.recall),
					},
				}
			})
			.collect();
	let overall = OverallClassMetrics {
		label: format_date_window_interval(overall.start_date, &date_window_interval, timezone),
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
							&date_window_interval,
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
		0
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
	let page_info = PageInfo {
		client_wasm_js_src: Some(client!()),
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
