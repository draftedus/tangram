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
	alert: Option<String>,
	name: String,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	intervals: Vec<NumberInterval>,
	overall: NumberOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberInterval {
	label: String,
	stats: Option<NumberStats>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberStats {
	max: f32,
	min: f32,
	mean: f32,
	p25: f32,
	p50: f32,
	p75: f32,
	std: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberOverall {
	absent_count: u64,
	invalid_count: u64,
	label: String,
	row_count: u64,
	stats: NumberOverallStats,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberOverallStats {
	production: Option<NumberStats>,
	training: NumberStats,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumProps {
	alert: Option<String>,
	name: String,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	intervals: Vec<EnumInterval>,
	overall: EnumOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumInterval {
	label: String,
	histogram: Vec<(String, u64)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumOverall {
	label: String,
	histogram: Vec<(String, EnumOverallHistogramEntry)>,
	row_count: u64,
	invalid_histogram: Option<Vec<(String, u64)>>,
	invalid_count: u64,
	absent_count: u64,
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
	alert: Option<String>,
	name: String,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	overall: TextOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TextOverall {
	label: String,
	token_histogram: Vec<(String, u64)>,
	row_count: u64,
	invalid_count: u64,
	absent_count: u64,
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
	let production_column_stats = get_production_column_stats(
		&mut db,
		&model,
		column_name,
		date_window,
		date_window_interval,
		timezone,
	)
	.await?;
	let train_row_count = match model {
		tangram_core::types::Model::Regressor(model) => model.row_count.into_option().unwrap(),
		tangram_core::types::Model::Classifier(model) => model.row_count.into_option().unwrap(),
		_ => unreachable!(),
	};
	let train_column_stats = match &model {
		tangram_core::types::Model::Classifier(model) => model
			.overall_column_stats
			.as_option()
			.unwrap()
			.into_iter()
			.find(|column| column.column_name() == column_name)
			.unwrap(),
		tangram_core::types::Model::Regressor(model) => model
			.overall_column_stats
			.as_option()
			.unwrap()
			.into_iter()
			.find(|column| column.column_name() == column_name)
			.unwrap(),
		tangram_core::types::Model::UnknownVariant(_, _, _) => unimplemented!(),
	};
	let inner = match train_column_stats {
		tangram_core::types::ColumnStats::Number(train_column_stats) => {
			Inner::Number(number_props(
				production_column_stats,
				train_column_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		tangram_core::types::ColumnStats::Enum(train_column_stats) => Inner::Enum(enum_props(
			production_column_stats,
			train_column_stats,
			train_row_count,
			date_window,
			date_window_interval,
			timezone,
		)),
		tangram_core::types::ColumnStats::Text(train_column_stats) => Inner::Text(text_props(
			production_column_stats,
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
	let overall = match get_production_stats_output.overall {
		ProductionColumnStatsOutput::Number(overall) => overall,
		_ => unreachable!(),
	};
	let overall = NumberOverall {
		absent_count: get_production_stats_output.overall.absent_count,
		invalid_count: get_production_stats_output.overall.invalid_count,
		label: format_date_window(
			get_production_stats_output.overall.start_date,
			date_window,
			timezone,
		),
		row_count: get_production_stats_output.overall.predictions_count,
		stats: NumberOverallStats {
			production: get_production_stats_output
				.overall
				.stats
				.as_ref()
				.map(|stats| NumberStats {
					max: stats.max,
					min: stats.min,
					mean: stats.mean,
					p25: stats.p25,
					p50: stats.p50,
					p75: stats.p75,
					std: stats.std,
				}),
			training: NumberStats {
				max: *train_column_stats.max.as_option().unwrap(),
				min: *train_column_stats.min.as_option().unwrap(),
				mean: *train_column_stats.mean.as_option().unwrap(),
				p25: *train_column_stats.p25.as_option().unwrap(),
				p50: *train_column_stats.p50.as_option().unwrap(),
				p75: *train_column_stats.p75.as_option().unwrap(),
				std: *train_column_stats.std.as_option().unwrap(),
			},
		},
	};
	let intervals = get_production_column_stats_output
		.intervals
		.into_iter()
		.map(|interval| NumberInterval {
			label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
			stats: interval.stats.map(|c| NumberStats {
				max: c.max,
				min: c.min,
				mean: c.mean,
				p25: c.p25,
				p50: c.p50,
				p75: c.p75,
				std: c.std,
			}),
		})
		.collect();
	NumberProps {
		// TODO
		alert: None,
		name: train_column_stats.column_name,
		date_window,
		date_window_interval,
		intervals,
		overall,
	}
}

fn enum_props(
	get_production_stats_output: GetProductionColumnStatsOutput,
	train_column_stats: tangram_core::types::EnumColumnStats,
	train_row_count: u64,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> EnumProps {
	let production_row_count = production_stats.overall.predictions_count;
	let histogram = production_stats
		.overall
		.histogram
		.into_iter()
		.zip(
			train_column_stats
				.histogram
				.into_option()
				.unwrap()
				.into_iter(),
		)
		.map(
			|((production_key, production_count), (train_key, training_count))| {
				if production_key != train_key {
					panic!();
				}
				(
					production_key,
					EnumOverallHistogramEntry {
						production_count,
						training_count,
						production_fraction: production_count.to_f32().unwrap()
							/ production_row_count.to_f32().unwrap(),
						training_fraction: training_count.to_f32().unwrap()
							/ train_row_count.to_f32().unwrap(),
					},
				)
			},
		)
		.collect();
	let overall = EnumOverall {
		absent_count: production_stats.overall.absent_count,
		invalid_count: production_stats.overall.invalid_count,
		label: format_date_window(production_stats.overall.start_date, date_window, timezone),
		row_count: production_stats.overall.predictions_count,
		histogram,
		invalid_histogram: production_stats.overall.invalid_histogram,
	};
	let intervals = production_stats
		.intervals
		.into_iter()
		.map(|interval| EnumInterval {
			label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
			histogram: interval.histogram,
		})
		.collect();
	EnumProps {
		alert: production_stats.overall.alert,
		name: production_stats.overall.column_name,
		date_window,
		date_window_interval,
		intervals,
		overall,
	}
}

fn text_props(
	get_production_stats_output: GetProductionColumnStatsOutput,
	_train_column_stats: tangram_core::types::TextColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> TextProps {
	let overall = TextOverall {
		absent_count: production_stats.overall.absent_count,
		invalid_count: production_stats.overall.invalid_count,
		label: format_date_window(production_stats.overall.start_date, date_window, timezone),
		row_count: production_stats.overall.predictions_count,
		token_histogram: production_stats.overall.token_histogram,
	};
	TextProps {
		alert: production_stats.overall.alert,
		name: production_stats.overall.column_name,
		date_window,
		date_window_interval,
		overall,
	}
}
