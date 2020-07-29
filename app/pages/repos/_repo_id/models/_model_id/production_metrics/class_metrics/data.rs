use crate::app::{
	cookies,
	error::Error,
	helpers::production_metrics,
	pages::repos::new::actions::get_repo_for_model,
	time::format_date_window_interval,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use chrono_tz::UTC;
use hyper::{header, Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use tangram::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionMetricsClassMetricsViewModel {
	id: String,
	title: String,
	class_metrics: Vec<ClassMetricsEntry>,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	classes: Vec<String>,
	overall: OverallClassMetrics,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetricsEntry {
	class_name: String,
	intervals: Vec<IntervalEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntervalEntry {
	label: String,
	f1_score: TrainingProductionMetrics,
	precision: TrainingProductionMetrics,
	recall: TrainingProductionMetrics,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallClassMetrics {
	class_metrics: Vec<OverallClassMetricsEntry>,
	label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallClassMetricsEntry {
	class_name: String,
	comparison: Comparison,
	confusion_matrix: ConfusionMatrix,
	f1_score: TrainingProductionMetrics,
	precision: TrainingProductionMetrics,
	recall: TrainingProductionMetrics,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Comparison {
	false_negative_fraction: TrainingProductionMetrics,
	false_positive_fraction: TrainingProductionMetrics,
	true_positive_fraction: TrainingProductionMetrics,
	true_negative_fraction: TrainingProductionMetrics,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfusionMatrix {
	false_negatives: Option<u64>,
	true_negatives: Option<u64>,
	true_positives: Option<u64>,
	false_positives: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingProductionMetrics {
	production: Option<f32>,
	training: f32,
}

pub async fn data(
	request: Request<Body>,
	context: Arc<Context>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	// parse the date window search param
	let date_window = search_params
		.as_ref()
		.and_then(|query| query.get("date_window"));
	let date_window = date_window.map(|dw| dw.as_str()).ok_or(Error::BadRequest)?;
	let date_window = match date_window {
		"today" => types::DateWindow::Today,
		"this_month" => types::DateWindow::ThisMonth,
		"this_year" => types::DateWindow::ThisYear,
		_ => return Err(Error::BadRequest.into()),
	};
	// choose the interval to use for the date window
	let date_window_interval = match date_window {
		types::DateWindow::Today => types::DateWindowInterval::Hourly,
		types::DateWindow::ThisMonth => types::DateWindowInterval::Daily,
		types::DateWindow::ThisYear => types::DateWindowInterval::Monthly,
	};
	// get the timezone
	let timezone = request
		.headers()
		.get(header::COOKIE)
		.and_then(|cookie_header_value| cookie_header_value.to_str().ok())
		.and_then(|cookie_header_value| cookies::parse(cookie_header_value).ok())
		.and_then(|cookies| cookies.get("tangram-timezone").cloned())
		.and_then(|timezone_str| timezone_str.parse().ok())
		.unwrap_or(UTC);

	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}

	// get the necessary data from the model
	let rows = db
		.query(
			"
				select
					id,
					title,
					created_at,
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram::types::Model::from_slice(&data)?;

	let production_metrics = production_metrics::get_production_metrics(
		&db,
		&model,
		date_window,
		date_window_interval,
		timezone,
	)
	.await?;

	let model = match model {
		tangram::types::Model::Classifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
	};
	let classes = model.classes();

	let types::ProductionMetricsResponse {
		overall, intervals, ..
	} = production_metrics;
	let overall_prediction_metrics =
		overall
			.prediction_metrics
			.map(|prediction_metrics| match prediction_metrics {
				types::PredictionMetrics::Regression(_) => unreachable!(),
				types::PredictionMetrics::Classification(prediction_metrics) => prediction_metrics,
			});

	let training_class_metrics = model
		.test_metrics
		.as_option()
		.unwrap()
		.class_metrics
		.as_option()
		.unwrap();

	let overall_class_metrics: Vec<OverallClassMetricsEntry> = training_class_metrics
		.iter()
		.zip(classes.iter())
		.enumerate()
		.map(|(class_index, (training_class_metrics, class_name))| {
			let production_class_metrics = overall_prediction_metrics
				.as_ref()
				.map(|prediction_metrics| &prediction_metrics.class_metrics[class_index]);

			let training_tn = training_class_metrics.true_negatives.as_option().unwrap();
			let training_fp = training_class_metrics.false_positives.as_option().unwrap();
			let training_tp = training_class_metrics.true_positives.as_option().unwrap();
			let training_fn = training_class_metrics.false_negatives.as_option().unwrap();
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
					training: *training_class_metrics.f1_score.as_option().unwrap(),
					production: production_class_metrics.map(|m| m.f1_score),
				},
				precision: TrainingProductionMetrics {
					production: production_class_metrics.map(|m| m.precision),
					training: *training_class_metrics.f1_score.as_option().unwrap(),
				},
				recall: TrainingProductionMetrics {
					training: *training_class_metrics.recall.as_option().unwrap(),
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
								types::PredictionMetrics::Regression(_) => unreachable!(),
								types::PredictionMetrics::Classification(prediction_metrics) => {
									prediction_metrics
								}
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
							training: *training_class_metrics[class_index]
								.f1_score
								.as_option()
								.unwrap(),
						},
						precision: TrainingProductionMetrics {
							production: production_precision,
							training: *training_class_metrics[class_index]
								.precision
								.as_option()
								.unwrap(),
						},
						recall: TrainingProductionMetrics {
							production: production_recall,
							training: *training_class_metrics[class_index]
								.precision
								.as_option()
								.unwrap(),
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

	let response = ProductionMetricsClassMetricsViewModel {
		id: id.to_string(),
		title: title.to_string(),
		class_metrics,
		date_window,
		date_window_interval,
		classes: classes.to_owned(),
		overall,
		repo: get_repo_for_model(&db, model_id).await?,
	};

	let response = serde_json::to_vec(&response)?;

	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}
