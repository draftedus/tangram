use crate::{
	error::Error,
	helpers::{
		model::{get_model, Model},
		production_stats,
		repos::{get_model_layout_info, ModelLayoutInfo},
		time::{format_date_window, format_date_window_interval},
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	types::{DateWindow, DateWindowInterval},
	Context,
};

use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use std::collections::BTreeMap;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, search_params).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/production_stats/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	model_id: String,
	overall_column_stats_table: Vec<OverallColumnStats>,
	prediction_count_chart: Vec<PredictionCountChartEntry>,
	prediction_stats_chart: PredictionStatsChart,
	prediction_stats_interval_chart: PredictionStatsIntervalChart,
	model_layout_info: ModelLayoutInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallColumnStats {
	absent_count: u64,
	invalid_count: u64,
	alert: Option<String>,
	name: String,
	column_type: ColumnType,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
enum PredictionStatsChart {
	Regression(RegressionChartEntry),
	Classification(ClassificationChartEntry),
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
enum PredictionStatsIntervalChart {
	Regression(Vec<RegressionChartEntry>),
	Classification(Vec<ClassificationChartEntry>),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PredictionCountChartEntry {
	count: u64,
	label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressionChartEntry {
	label: String,
	quantiles: ProductionTrainingQuantiles,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassificationChartEntry {
	label: String,
	histogram: ProductionTrainingHistogram,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionTrainingHistogram {
	production: Vec<(String, u64)>,
	training: Vec<(String, u64)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionTrainingQuantiles {
	production: Option<Quantiles>,
	training: Quantiles,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Quantiles {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
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
	let date_window = date_window.map_or("this_month", |dw| dw.as_str());
	let date_window = match date_window {
		"today" => DateWindow::Today,
		"this_month" => DateWindow::ThisMonth,
		"this_year" => DateWindow::ThisYear,
		_ => return Err(Error::BadRequest.into()),
	};
	// choose the interval to use for the date window
	let date_window_interval = match date_window {
		DateWindow::Today => DateWindowInterval::Hourly,
		DateWindow::ThisMonth => DateWindowInterval::Daily,
		DateWindow::ThisYear => DateWindowInterval::Monthly,
	};
	// get the timezone
	let timezone = get_timezone(&request);

	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}

	let Model { data, id } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;

	let production_stats = production_stats::get_production_stats(
		&mut db,
		&model,
		date_window,
		date_window_interval,
		timezone,
	)
	.await?;

	let target_column_stats = match model {
		tangram_core::types::Model::Classifier(model) => {
			model.overall_target_column_stats.into_option().unwrap()
		}
		tangram_core::types::Model::Regressor(model) => {
			model.overall_target_column_stats.into_option().unwrap()
		}
		tangram_core::types::Model::UnknownVariant(_, _, _) => unimplemented!(),
	};

	let overall_column_stats_table = production_stats
		.overall
		.column_stats
		.iter()
		.map(|column_stats| match column_stats {
			ProductionColumnStatsOutput::Unknown(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: column_stats.alert.to_owned(),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Unknown,
			},
			ProductionColumnStatsOutput::Text(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: column_stats.alert.to_owned(),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Text,
			},
			ProductionColumnStatsOutput::Number(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: column_stats.alert.to_owned(),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Number,
			},
			ProductionColumnStatsOutput::Enum(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: column_stats.alert.to_owned(),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Enum,
			},
		})
		.collect();

	let prediction_count_chart = production_stats
		.intervals
		.iter()
		.map(|interval| PredictionCountChartEntry {
			count: interval.predictions_count,
			label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
		})
		.collect();

	let overall_production_stats = production_stats.overall;
	let prediction_stats_chart = match overall_production_stats.prediction_stats {
		ProductionPredictionStatsOutput::Regression(prediction_stats) => {
			let target_column_stats = target_column_stats.as_number().unwrap();
			PredictionStatsChart::Regression(RegressionChartEntry {
				label: format_date_window(
					overall_production_stats.start_date,
					date_window,
					timezone,
				),
				quantiles: compute_production_training_quantiles(
					target_column_stats,
					&prediction_stats,
				),
			})
		}
		ProductionPredictionStatsOutput::Classification(prediction_stats) => {
			let target_column_stats = target_column_stats.as_enum().unwrap();
			PredictionStatsChart::Classification(ClassificationChartEntry {
				label: format_date_window(
					overall_production_stats.start_date,
					date_window,
					timezone,
				),
				histogram: ProductionTrainingHistogram {
					production: prediction_stats.histogram,
					training: target_column_stats.histogram.as_option().unwrap().clone(),
				},
			})
		}
	};
	let interval_production_stats = production_stats.intervals;
	let prediction_stats_interval_chart = match &interval_production_stats[0].prediction_stats {
		ProductionPredictionStatsOutput::Regression(_) => PredictionStatsIntervalChart::Regression(
			interval_production_stats
				.iter()
				.map(|interval_production_stats| {
					match &interval_production_stats.prediction_stats {
						ProductionPredictionStatsOutput::Regression(interval_prediction_stats) => {
							let target_column_stats = target_column_stats.as_number().unwrap();
							RegressionChartEntry {
								label: format_date_window_interval(
									interval_production_stats.start_date,
									date_window_interval,
									timezone,
								),
								quantiles: compute_production_training_quantiles(
									target_column_stats,
									interval_prediction_stats,
								),
							}
						}
						_ => unreachable!(),
					}
				})
				.collect(),
		),
		ProductionPredictionStatsOutput::Classification(_) => {
			PredictionStatsIntervalChart::Classification(
				interval_production_stats
					.into_iter()
					.map(|interval_production_stats| {
						match interval_production_stats.prediction_stats {
							ProductionPredictionStatsOutput::Classification(prediction_stats) => {
								let target_column_stats = target_column_stats.as_enum().unwrap();
								ClassificationChartEntry {
									label: format_date_window_interval(
										interval_production_stats.start_date,
										date_window_interval,
										timezone,
									),
									histogram: ProductionTrainingHistogram {
										production: prediction_stats.histogram,
										training: target_column_stats
											.histogram
											.as_option()
											.unwrap()
											.clone(),
									},
								}
							}
							_ => unreachable!(),
						}
					})
					.collect(),
			)
		}
	};

	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;

	Ok(Props {
		overall_column_stats_table,
		model_id: id.to_string(),
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		model_layout_info,
	})
}

fn compute_production_training_quantiles(
	target_column_stats: &tangram_core::types::NumberColumnStats,
	prediction_stats: &RegressionProductionPredictionStats,
) -> ProductionTrainingQuantiles {
	ProductionTrainingQuantiles {
		production: if let Some(prediction_stats) = &prediction_stats.stats {
			Some(Quantiles {
				max: prediction_stats.max,
				p25: prediction_stats.p25,
				p50: prediction_stats.p50,
				p75: prediction_stats.p75,
				min: prediction_stats.min,
			})
		} else {
			None
		},
		training: Quantiles {
			max: *target_column_stats.max.as_option().unwrap(),
			min: *target_column_stats.min.as_option().unwrap(),
			p25: *target_column_stats.p25.as_option().unwrap(),
			p50: *target_column_stats.p50.as_option().unwrap(),
			p75: *target_column_stats.p75.as_option().unwrap(),
		},
	}
}
