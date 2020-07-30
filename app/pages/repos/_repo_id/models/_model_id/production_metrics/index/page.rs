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
use serde::Serialize;
use std::collections::BTreeMap;
use tangram::id::Id;

pub async fn page(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, search_params).await?;
	let html = context
		.pinwheel
		.render(
			"/repos/_repoId_/models/_modelId_/production_metrics/",
			props,
		)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	title: String,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum Inner {
	Regressor(RegressorProductionMetricsOverviewViewModel),
	Classifier(ClassifierProductionMetricsOverviewViewModel),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorProductionMetricsOverviewViewModel {
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	mse_chart: MSEChart,
	overall: RegressionProductionMetrics,
	true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrueValuesCountChartEntry {
	label: String,
	count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MSEChart {
	data: Vec<MSEChartEntry>,
	training_mse: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MSEChartEntry {
	label: String,
	mse: Option<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressionProductionMetrics {
	mse: TrainingProductionMetrics,
	rmse: TrainingProductionMetrics,
	true_values_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingProductionMetrics {
	production: Option<f32>,
	training: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassifierProductionMetricsOverviewViewModel {
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	overall: ClassificationOverallProductionMetrics,
	id: String,
	accuracy_chart: AccuracyChart,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccuracyChart {
	data: Vec<AccuracyChartEntry>,
	training_accuracy: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccuracyChartEntry {
	accuracy: Option<f32>,
	label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassificationOverallProductionMetrics {
	accuracy: TrainingProductionMetrics,
	class_metrics_table: Vec<ClassMetricsTableEntry>,
	true_values_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetricsTableEntry {
	class_name: String,
	precision: TrainingProductionMetrics,
	recall: TrainingProductionMetrics,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
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

	let inner = match &model {
		tangram::types::Model::Regressor(model) => {
			let training_metrics = model.test_metrics.as_option().unwrap();
			let true_values_count = production_metrics.overall.true_values_count;
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						types::PredictionMetrics::Regression(metrics) => metrics,
						_ => unreachable!(),
					});

			let overall = RegressionProductionMetrics {
				mse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.mse),
					training: *training_metrics.mse.as_option().unwrap(),
				},
				rmse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.rmse),
					training: *training_metrics.rmse.as_option().unwrap(),
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
								if let types::PredictionMetrics::Regression(predicion_metrics) =
									prediction_metrics
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
					training_mse: *training_metrics.mse.as_option().unwrap(),
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

			Inner::Regressor(RegressorProductionMetricsOverviewViewModel {
				date_window,
				date_window_interval,
				mse_chart,
				overall,
				true_values_count_chart,
			})
		}
		tangram::types::Model::Classifier(model) => {
			let training_metrics = model.test_metrics.as_option().unwrap();
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						types::PredictionMetrics::Classification(metrics) => metrics,
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
									if let types::PredictionMetrics::Classification(
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
					training_accuracy: *training_metrics.accuracy.as_option().unwrap(),
				}
			};

			let true_values_count = production_metrics.overall.true_values_count;
			let training_class_metrics = training_metrics.class_metrics.as_option().unwrap();

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
							training: *training_class_metrics.precision.as_option().unwrap(),
							production: precision,
						},
						recall: TrainingProductionMetrics {
							training: *training_class_metrics.recall.as_option().unwrap(),
							production: recall,
						},
						class_name: class_name.to_owned(),
					}
				})
				.collect();

			let overall = ClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: *training_metrics.accuracy.as_option().unwrap(),
				},
				class_metrics_table,
				true_values_count,
			};

			Inner::Classifier(ClassifierProductionMetricsOverviewViewModel {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: id.to_string(),
				accuracy_chart,
				overall,
			})
		}
		_ => unimplemented!(),
	};

	let repo = get_repo_for_model(&db, model_id).await?;
	db.commit().await?;

	Ok(Props {
		id: id.to_string(),
		inner,
		title,
		repo,
	})
}
