use crate::{
	common::{
		date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		production_stats::get_production_stats,
		time::{format_date_window, format_date_window_interval},
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	production_stats::{
		ProductionColumnStatsOutput, ProductionPredictionStatsOutput,
		RegressionProductionPredictionStatsOutput,
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallColumnStats {
	absent_count: u64,
	invalid_count: u64,
	alert: Option<String>,
	name: String,
	column_type: ColumnType,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "data")]
enum PredictionStatsChart {
	#[serde(rename = "regression")]
	Regression(RegressionChartEntry),
	#[serde(rename = "classification")]
	MulticlassClassification(MulticlassClassificationChartEntry),
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "data")]
enum PredictionStatsIntervalChart {
	#[serde(rename = "regression")]
	Regression(Vec<RegressionChartEntry>),
	#[serde(rename = "classification")]
	MulticlassClassification(Vec<MulticlassClassificationChartEntry>),
}

#[derive(serde::Serialize)]
enum ColumnType {
	#[serde(rename = "unknown")]
	Unknown,
	#[serde(rename = "number")]
	Number,
	#[serde(rename = "enum")]
	Enum,
	#[serde(rename = "text")]
	Text,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PredictionCountChartEntry {
	count: u64,
	label: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressionChartEntry {
	label: String,
	quantiles: ProductionTrainingQuantiles,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassificationChartEntry {
	label: String,
	histogram: ProductionTrainingHistogram,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionTrainingHistogram {
	production: Vec<(String, u64)>,
	training: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionTrainingQuantiles {
	production: Option<Quantiles>,
	training: Quantiles,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Quantiles {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
}

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

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
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
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let model = get_model(&mut db, model_id).await?;
	let production_stats =
		get_production_stats(&mut db, &model, date_window, date_window_interval, timezone).await?;
	let target_column_stats = match model {
		tangram_core::model::Model::Regressor(model) => model.overall_target_column_stats,
		tangram_core::model::Model::BinaryClassifier(model) => model.overall_target_column_stats,
		tangram_core::model::Model::MulticlassClassifier(model) => {
			model.overall_target_column_stats
		}
	};
	let overall_column_stats_table = production_stats
		.overall
		.column_stats
		.iter()
		.map(|column_stats| match column_stats {
			ProductionColumnStatsOutput::Unknown(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: None,
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Unknown,
			},
			ProductionColumnStatsOutput::Text(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: None,
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Text,
			},
			ProductionColumnStatsOutput::Number(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: None,
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Number,
			},
			ProductionColumnStatsOutput::Enum(column_stats) => OverallColumnStats {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				alert: None,
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Enum,
			},
		})
		.collect();
	let prediction_count_chart = production_stats
		.intervals
		.iter()
		.map(|interval| PredictionCountChartEntry {
			count: interval.row_count,
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
		ProductionPredictionStatsOutput::BinaryClassification(prediction_stats)
		| ProductionPredictionStatsOutput::MulticlassClassification(prediction_stats) => {
			let target_column_stats = target_column_stats.as_enum().unwrap();
			PredictionStatsChart::MulticlassClassification(MulticlassClassificationChartEntry {
				label: format_date_window(
					overall_production_stats.start_date,
					date_window,
					timezone,
				),
				histogram: ProductionTrainingHistogram {
					production: prediction_stats.histogram,
					training: target_column_stats.histogram.clone(),
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
		ProductionPredictionStatsOutput::BinaryClassification(_)
		| ProductionPredictionStatsOutput::MulticlassClassification(_) => {
			PredictionStatsIntervalChart::MulticlassClassification(
				interval_production_stats
					.into_iter()
					.map(|interval_production_stats| {
						match interval_production_stats.prediction_stats {
							ProductionPredictionStatsOutput::MulticlassClassification(
								prediction_stats,
							) => {
								let target_column_stats = target_column_stats.as_enum().unwrap();
								MulticlassClassificationChartEntry {
									label: format_date_window_interval(
										interval_production_stats.start_date,
										date_window_interval,
										timezone,
									),
									histogram: ProductionTrainingHistogram {
										production: prediction_stats.histogram,
										training: target_column_stats.histogram.clone(),
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
		model_id: model_id.to_string(),
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		model_layout_info,
	})
}

fn compute_production_training_quantiles(
	target_column_stats: &tangram_core::model::NumberColumnStats,
	prediction_stats: &RegressionProductionPredictionStatsOutput,
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
			max: target_column_stats.max,
			min: target_column_stats.min,
			p25: target_column_stats.p25,
			p50: target_column_stats.p50,
			p75: target_column_stats.p75,
		},
	}
}

// TODO

// const LARGE_ABSENT_RATIO_THRESHOLD: f32 = 0.1;
// const LARGE_INVALID_RATIO_THRESHOLD: f32 = 0.1;
// let invalid_ratio = self.invalid_count.to_f32().unwrap() / self.count.to_f32().unwrap();
// let absent_ratio = self.absent_count.to_f32().unwrap() / self.count.to_f32().unwrap();
// let alert = alert_message(invalid_ratio, absent_ratio);

// fn alert_message(invalid_ratio: f32, absent_ratio: f32) -> Option<String> {
// 	if invalid_ratio > LARGE_INVALID_RATIO_THRESHOLD {
// 		if absent_ratio > LARGE_ABSENT_RATIO_THRESHOLD {
// 			Some("High Invalid and Absent Count".into())
// 		} else {
// 			Some("High Invalid Count".into())
// 		}
// 	} else if absent_ratio > LARGE_ABSENT_RATIO_THRESHOLD {
// 		Some("High Absent Count".into())
// 	} else {
// 		None
// 	}
// }
