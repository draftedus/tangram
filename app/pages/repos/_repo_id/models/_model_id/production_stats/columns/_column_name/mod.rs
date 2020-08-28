use crate::{
	common::{
		date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
		model::{get_model, Model},
		production_stats::{get_production_column_stats, GetProductionColumnStatsOutput},
		repos::{get_model_layout_info, ModelLayoutInfo},
		time::{format_date_window, format_date_window_interval},
		timezone::get_timezone,
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	production_stats::ProductionColumnStatsOutput,
	Context,
};
use anyhow::Result;
use chrono_tz::Tz;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use serde::Serialize;
use std::collections::BTreeMap;
use tangram_core::id::Id;

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	date_window: DateWindow,
	column_name: String,
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum Inner {
	Number(NumberProps),
	Enum(EnumProps),
	Text(TextProps),
}

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntervalBoxChartDataPoint {
	label: String,
	stats: Option<IntervalBoxChartDataPointStats>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntervalBoxChartDataPointStats {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallBoxChartData {
	production: Option<OverallBoxChartDataStats>,
	training: OverallBoxChartDataStats,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallBoxChartDataStats {
	max: f32,
	min: f32,
	p25: f32,
	p50: f32,
	p75: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberTrainingProductionComparison {
	production: Option<f32>,
	training: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumProps {
	alert: Option<String>,
	absent_count: u64,
	column_name: String,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	interval_chart_data: Vec<EnumIntervalChartDataPoint>,
	invalid_count: u64,
	overall_chart_data: Vec<(String, EnumOverallHistogramEntry)>,
	overall_invalid_chart_data: Option<Vec<(String, u64)>>,
	row_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumIntervalChartDataPoint {
	label: String,
	histogram: Vec<(String, u64)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumOverallHistogramEntry {
	production_count: u64,
	production_fraction: f32,
	training_count: u64,
	training_fraction: f32,
}

#[derive(Serialize)]
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

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
	let (date_window, date_window_interval) = get_date_window_and_interval(&search_params)?;
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
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	let get_production_column_stats_output = get_production_column_stats(
		&mut db,
		&model,
		column_name,
		date_window,
		date_window_interval,
		timezone,
	)
	.await?;
	let train_row_count = match &model {
		tangram_core::types::Model::Regressor(model) => *model.row_count.as_option().unwrap(),
		tangram_core::types::Model::Classifier(model) => *model.row_count.as_option().unwrap(),
		_ => unreachable!(),
	};
	let train_column_stats = match &model {
		tangram_core::types::Model::Classifier(model) => model
			.overall_column_stats
			.as_option()
			.unwrap()
			.iter()
			.find(|column| column.column_name() == column_name)
			.unwrap(),
		tangram_core::types::Model::Regressor(model) => model
			.overall_column_stats
			.as_option()
			.unwrap()
			.iter()
			.find(|column| column.column_name() == column_name)
			.unwrap(),
		tangram_core::types::Model::UnknownVariant(_, _, _) => unimplemented!(),
	};
	let inner = match train_column_stats {
		tangram_core::types::ColumnStats::Number(train_column_stats) => {
			Inner::Number(number_props(
				get_production_column_stats_output,
				train_column_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		tangram_core::types::ColumnStats::Enum(train_column_stats) => Inner::Enum(enum_props(
			get_production_column_stats_output,
			train_column_stats,
			train_row_count,
			date_window,
			date_window_interval,
			timezone,
		)),
		tangram_core::types::ColumnStats::Text(train_column_stats) => Inner::Text(text_props(
			get_production_column_stats_output,
			train_column_stats,
			date_window,
			date_window_interval,
		)),
		_ => return Err(Error::BadRequest.into()),
	};
	db.commit().await?;
	Ok(Props {
		date_window,
		column_name: column_name.to_owned(),
		id: id.to_string(),
		inner,
		model_layout_info,
	})
}

fn number_props(
	get_production_stats_output: GetProductionColumnStatsOutput,
	train_column_stats: &tangram_core::types::NumberColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> NumberProps {
	// let overall = match get_production_stats_output.overall {
	// 	ProductionColumnStatsOutput::Number(overall) => overall,
	// 	_ => unreachable!(),
	// };
	// let overall = NumberOverall {
	// 	label: format_date_window(
	// 		get_production_stats_output.overall.start_date,
	// 		date_window,
	// 		timezone,
	// 	),
	// 	stats: NumberOverallStats {
	// 		production: get_production_stats_output
	// 			.overall
	// 			.stats
	// 			.as_ref()
	// 			.map(|stats| NumberStats {
	// 				max: stats.max,
	// 				min: stats.min,
	// 				mean: stats.mean,
	// 				p25: stats.p25,
	// 				p50: stats.p50,
	// 				p75: stats.p75,
	// 				std: stats.std,
	// 			}),
	// 		training: NumberStats {
	// 			max: *train_column_stats.max.as_option().unwrap(),
	// 			min: *train_column_stats.min.as_option().unwrap(),
	// 			mean: *train_column_stats.mean.as_option().unwrap(),
	// 			p25: *train_column_stats.p25.as_option().unwrap(),
	// 			p50: *train_column_stats.p50.as_option().unwrap(),
	// 			p75: *train_column_stats.p75.as_option().unwrap(),
	// 			std: *train_column_stats.std.as_option().unwrap(),
	// 		},
	// 	},
	// };
	// let intervals = get_production_column_stats_output
	// 	.intervals
	// 	.into_iter()
	// 	.map(|interval| NumberInterval {
	// 		label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
	// 		stats: interval.stats.map(|c| NumberStats {
	// 			max: c.max,
	// 			min: c.min,
	// 			mean: c.mean,
	// 			p25: c.p25,
	// 			p50: c.p50,
	// 			p75: c.p75,
	// 			std: c.std,
	// 		}),
	// 	})
	// 	.collect();
	// NumberProps {
	// 	// TODO
	// 	alert: None,
	// 	column_name: train_column_stats.column_name,
	// 	date_window,
	// 	date_window_interval,
	// 	intervals,
	// 	overall,
	// }
	todo!()
}

fn enum_props(
	get_production_stats_output: GetProductionColumnStatsOutput,
	train_column_stats: &tangram_core::types::EnumColumnStats,
	train_row_count: u64,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> EnumProps {
	// let production_row_count = production_stats.overall.predictions_count;
	// let histogram = production_stats
	// 	.overall
	// 	.histogram
	// 	.into_iter()
	// 	.zip(
	// 		train_column_stats
	// 			.histogram
	// 			.into_option()
	// 			.unwrap()
	// 			.into_iter(),
	// 	)
	// 	.map(
	// 		|((production_key, production_count), (train_key, training_count))| {
	// 			if production_key != train_key {
	// 				panic!();
	// 			}
	// 			(
	// 				production_key,
	// 				EnumOverallHistogramEntry {
	// 					production_count,
	// 					training_count,
	// 					production_fraction: production_count.to_f32().unwrap()
	// 						/ production_row_count.to_f32().unwrap(),
	// 					training_fraction: training_count.to_f32().unwrap()
	// 						/ train_row_count.to_f32().unwrap(),
	// 				},
	// 			)
	// 		},
	// 	)
	// 	.collect();
	// let overall = EnumOverallChartData {
	// 	// label: format_date_window(production_stats.overall.start_date, date_window, timezone),
	// 	histogram,
	// };
	// let intervals = production_stats
	// 	.intervals
	// 	.into_iter()
	// 	.map(|interval| EnumIntervalChartDataPoint {
	// 		label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
	// 		histogram: interval.histogram,
	// 	})
	// 	.collect();
	// EnumProps {
	// 	invalid_histogram: production_stats.overall.invalid_histogram,
	// 	row_count: production_stats.overall.predictions_count,
	// 	absent_count: production_stats.overall.absent_count,
	// 	invalid_count: production_stats.overall.invalid_count,
	// 	alert: production_stats.overall.alert,
	// 	column_name: production_stats.overall.column_name,
	// 	date_window,
	// 	date_window_interval,
	// 	interval_chart_data: intervals,
	// 	overall,
	// }
	todo!()
}

fn text_props(
	get_production_column_stats_output: GetProductionColumnStatsOutput,
	_train_column_stats: &tangram_core::types::TextColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
) -> TextProps {
	let overall_production_column_stats_output = match get_production_column_stats_output.overall {
		ProductionColumnStatsOutput::Text(p) => p,
		_ => unreachable!(),
	};
	TextProps {
		alert: None,
		row_count: 0,
		absent_count: overall_production_column_stats_output.absent_count,
		invalid_count: overall_production_column_stats_output.invalid_count,
		column_name: overall_production_column_stats_output.column_name,
		date_window,
		date_window_interval,
		overall_token_histogram: overall_production_column_stats_output.token_histogram,
	}
}
