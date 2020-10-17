use crate::{
	common::{
		date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		production_stats::{get_production_stats, GetProductionStatsOutput},
		time::format_date_window_interval,
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	production_stats::ProductionColumnStatsOutput,
	Context,
};
use anyhow::Result;
use chrono_tz::Tz;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	date_window: DateWindow,
	column_name: String,
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
enum Inner {
	Number(NumberProps),
	Enum(EnumProps),
	Text(TextProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberProps {
	absent_count: u64,
	alert: Option<String>,
	column_name: String,
	date_window_interval: DateWindowInterval,
	date_window: DateWindow,
	interval_box_chart_data: Vec<IntervalBoxChartDataPoint>,
	invalid_count: u64,
	max_comparison: NumberTrainingProductionComparison,
	mean_comparison: NumberTrainingProductionComparison,
	min_comparison: NumberTrainingProductionComparison,
	overall_box_chart_data: OverallBoxChartData,
	row_count: u64,
	std_comparison: NumberTrainingProductionComparison,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntervalBoxChartDataPoint {
	label: String,
	stats: Option<IntervalBoxChartDataPointStats>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntervalBoxChartDataPointStats {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallBoxChartData {
	production: Option<OverallBoxChartDataStats>,
	training: OverallBoxChartDataStats,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallBoxChartDataStats {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberTrainingProductionComparison {
	production: Option<f32>,
	training: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumProps {
	alert: Option<String>,
	absent_count: u64,
	column_name: String,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	invalid_count: u64,
	overall_chart_data: Vec<(String, EnumOverallHistogramEntry)>,
	overall_invalid_chart_data: Option<Vec<(String, u64)>>,
	row_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumIntervalChartDataPoint {
	label: String,
	histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumOverallHistogramEntry {
	production_count: u64,
	production_fraction: f32,
	training_count: u64,
	training_fraction: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TextProps {
	absent_count: u64,
	alert: Option<String>,
	column_name: String,
	date_window_interval: DateWindowInterval,
	date_window: DateWindow,
	invalid_count: u64,
	overall_token_histogram: Vec<(String, u64)>,
	row_count: u64,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, column_name, search_params).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_stats/columns/_column_name",
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
	column_name: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
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
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let model = get_model(&mut db, model_id).await?;
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	let get_production_stats_output =
		get_production_stats(&mut db, &model, date_window, date_window_interval, timezone).await?;
	let train_row_count = match &model {
		tangram_core::model::Model::Regressor(model) => model.train_row_count,
		tangram_core::model::Model::BinaryClassifier(model) => model.train_row_count,
		tangram_core::model::Model::MulticlassClassifier(model) => model.train_row_count,
	};
	let overall_column_stats = match &model {
		tangram_core::model::Model::Regressor(model) => &model.overall_column_stats,
		tangram_core::model::Model::BinaryClassifier(model) => &model.overall_column_stats,
		tangram_core::model::Model::MulticlassClassifier(model) => &model.overall_column_stats,
	};
	let train_column_stats = overall_column_stats
		.iter()
		.find(|column| column.column_name() == column_name)
		.unwrap();
	let inner = match train_column_stats {
		tangram_core::model::ColumnStats::Number(train_column_stats) => {
			Inner::Number(number_props(
				get_production_stats_output,
				train_column_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		tangram_core::model::ColumnStats::Enum(train_column_stats) => Inner::Enum(enum_props(
			get_production_stats_output,
			train_column_stats,
			train_row_count,
			date_window,
			date_window_interval,
			timezone,
		)),
		tangram_core::model::ColumnStats::Text(train_column_stats) => Inner::Text(text_props(
			get_production_stats_output,
			train_column_stats,
			date_window,
			date_window_interval,
			timezone,
		)),
		_ => return Err(Error::BadRequest.into()),
	};
	db.commit().await?;
	Ok(Props {
		date_window,
		column_name: column_name.to_owned(),
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}

fn number_props(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: &tangram_core::model::NumberColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> NumberProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Number(s) => s,
		_ => unreachable!(),
	};
	let overall_box_chart_data = OverallBoxChartData {
		production: overall
			.stats
			.as_ref()
			.map(|stats| OverallBoxChartDataStats {
				max: stats.max,
				min: stats.min,
				p25: stats.p25,
				p50: stats.p50,
				p75: stats.p75,
			}),
		training: OverallBoxChartDataStats {
			max: train_column_stats.max,
			min: train_column_stats.min,
			p25: train_column_stats.p25,
			p50: train_column_stats.p50,
			p75: train_column_stats.p75,
		},
	};
	let interval_box_chart_data = get_production_stats_output
		.intervals
		.iter()
		.map(|interval| {
			let production_column_stats = interval
				.column_stats
				.iter()
				.find(|production_column_stats| {
					production_column_stats.column_name() == train_column_stats.column_name
				})
				.unwrap();
			let production_column_stats = match production_column_stats {
				ProductionColumnStatsOutput::Number(s) => s,
				_ => unreachable!(),
			};
			IntervalBoxChartDataPoint {
				label: format_date_window_interval(
					interval.start_date,
					date_window_interval,
					timezone,
				),
				stats: production_column_stats.stats.as_ref().map(|c| {
					IntervalBoxChartDataPointStats {
						max: c.max,
						min: c.min,
						p25: c.p25,
						p50: c.p50,
						p75: c.p75,
					}
				}),
			}
		})
		.collect();
	let min_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.min),
		training: train_column_stats.min,
	};
	let max_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.max),
		training: train_column_stats.max,
	};
	let mean_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.mean),
		training: train_column_stats.mean,
	};
	let std_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.std),
		training: train_column_stats.std,
	};
	NumberProps {
		absent_count: overall.absent_count,
		alert: None,
		column_name: train_column_stats.column_name.clone(),
		date_window_interval,
		date_window,
		interval_box_chart_data,
		invalid_count: overall.invalid_count,
		max_comparison,
		mean_comparison,
		min_comparison,
		overall_box_chart_data,
		row_count: get_production_stats_output.overall.row_count,
		std_comparison,
	}
}

fn enum_props(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: &tangram_core::model::EnumColumnStats,
	train_row_count: u64,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	_timezone: Tz,
) -> EnumProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Enum(s) => s,
		_ => unreachable!(),
	};
	let production_row_count = get_production_stats_output.overall.row_count;
	let overall_chart_data = overall
		.histogram
		.iter()
		.zip(train_column_stats.histogram.iter())
		.map(
			|((production_enum_option, production_count), (_, training_count))| {
				(
					production_enum_option.clone(),
					EnumOverallHistogramEntry {
						production_count: *production_count,
						training_count: *training_count,
						production_fraction: production_count.to_f32().unwrap()
							/ production_row_count.to_f32().unwrap(),
						training_fraction: training_count.to_f32().unwrap()
							/ train_row_count.to_f32().unwrap(),
					},
				)
			},
		)
		.collect();
	EnumProps {
		absent_count: overall.absent_count,
		alert: None,
		column_name: overall.column_name.clone(),
		date_window_interval,
		date_window,
		invalid_count: overall.invalid_count,
		overall_invalid_chart_data: overall.invalid_histogram.clone(),
		row_count: production_row_count,
		overall_chart_data,
	}
}

fn text_props(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: &tangram_core::model::TextColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	_timezone: Tz,
) -> TextProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Text(s) => s,
		_ => unreachable!(),
	};
	let overall_token_histogram = overall
		.token_histogram
		.iter()
		.map(|(token, count)| (token.to_string(), *count))
		.collect();
	TextProps {
		alert: None,
		row_count: get_production_stats_output.overall.row_count,
		absent_count: overall.absent_count,
		invalid_count: overall.invalid_count,
		column_name: overall.column_name.clone(),
		date_window,
		date_window_interval,
		overall_token_histogram,
	}
}
