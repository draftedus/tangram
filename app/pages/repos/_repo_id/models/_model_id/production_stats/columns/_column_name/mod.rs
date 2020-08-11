use crate::Context;
use crate::{
	cookies,
	error::Error,
	helpers::production_stats,
	helpers::{
		model::{get_model, Model},
		repos::get_model_layout_info,
	},
	time::{format_date_window, format_date_window_interval},
	types,
	user::{authorize_user, authorize_user_for_model},
};
use anyhow::Result;
use chrono_tz::{Tz, UTC};
use hyper::{header, Body, Request, Response, StatusCode};
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
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	date_window: types::DateWindow,
	column_name: String,
	id: String,
	inner: Inner,
	title: String,
	model_layout_info: types::ModelLayoutInfo,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum Inner {
	Number(NumberColumnStatsViewModel),
	Enum(EnumColumnStatsViewModel),
	Text(TextColumnStatsViewModel),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumColumnStatsViewModel {
	alert: Option<String>,
	name: String,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	intervals: Vec<EnumColumnStatsInterval>,
	overall: EnumColumnStatsOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumColumnStatsInterval {
	label: String,
	histogram: Vec<(String, u64)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumColumnStatsOverall {
	label: String,
	histogram: Vec<(String, OverallEnumColumnStatsEntry)>,
	row_count: u64,
	invalid_histogram: Option<Vec<(String, u64)>>,
	invalid_count: u64,
	absent_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverallEnumColumnStatsEntry {
	production_count: u64,
	production_fraction: f32,
	training_count: u64,
	training_fraction: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberColumnStatsViewModel {
	alert: Option<String>,
	name: String,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	intervals: Vec<NumberColumnStatsInterval>,
	overall: NumberColumnStatsOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberColumnStatsInterval {
	label: String,
	stats: Option<NumberColumnStats>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberColumnStatsOverall {
	absent_count: u64,
	invalid_count: u64,
	label: String,
	row_count: u64,
	stats: ProductionTrainingStats,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionTrainingStats {
	production: Option<NumberColumnStats>,
	training: NumberColumnStats,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NumberColumnStats {
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
struct TextColumnStatsViewModel {
	alert: Option<String>,
	name: String,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	overall: TextColumnStatsOverall,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TextColumnStatsOverall {
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
	// parse the date window search param
	let date_window = search_params
		.as_ref()
		.and_then(|query| query.get("date_window"));
	let date_window = date_window.map_or("this_month", |dw| dw.as_str());
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

	let Model { title, data, id } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;

	let production_stats = production_stats::get_production_column_stats(
		&mut db,
		&model,
		column_name,
		date_window,
		date_window_interval,
		timezone,
	)
	.await?;

	let (train_column_stats, train_row_count) = match model {
		tangram_core::types::Model::Classifier(model) => {
			let overall_column_stats = model.overall_column_stats.into_option().unwrap();
			(
				overall_column_stats
					.into_iter()
					.find(|column| column.column_name() == column_name),
				model.row_count.into_option().unwrap(),
			)
		}
		tangram_core::types::Model::Regressor(model) => {
			let overall_column_stats = model.overall_column_stats.into_option().unwrap();
			(
				overall_column_stats
					.into_iter()
					.find(|column| column.column_name() == column_name),
				model.row_count.into_option().unwrap(),
			)
		}
		tangram_core::types::Model::UnknownVariant(_, _, _) => unimplemented!(),
	};

	let inner = match (train_column_stats.unwrap(), production_stats) {
		(
			tangram_core::types::ColumnStats::Number(train_column_stats),
			types::ProductionStatsSingleColumnResponse::Number(production_stats),
		) => Inner::Number(build_production_column_number_stats(
			production_stats,
			train_column_stats,
			date_window,
			date_window_interval,
			timezone,
		)),
		(
			tangram_core::types::ColumnStats::Enum(train_column_stats),
			types::ProductionStatsSingleColumnResponse::Enum(production_stats),
		) => Inner::Enum(build_production_column_enum_stats(
			production_stats,
			train_column_stats,
			train_row_count,
			date_window,
			date_window_interval,
			timezone,
		)),
		(
			tangram_core::types::ColumnStats::Text(train_column_stats),
			types::ProductionStatsSingleColumnResponse::Text(production_stats),
		) => Inner::Text(build_production_column_text_stats(
			production_stats,
			train_column_stats,
			date_window,
			date_window_interval,
			timezone,
		)),
		(_, _) => return Err(Error::BadRequest.into()),
	};

	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;

	Ok(Props {
		date_window,
		column_name: column_name.to_owned(),
		id: id.to_string(),
		inner,
		title,
		model_layout_info,
	})
}

fn build_production_column_number_stats(
	production_stats: types::NumberProductionStatsSingleColumnResponse,
	train_column_stats: tangram_core::types::NumberColumnStats,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> NumberColumnStatsViewModel {
	let overall =
		NumberColumnStatsOverall {
			absent_count: production_stats.overall.absent_count,
			invalid_count: production_stats.overall.invalid_count,
			label: format_date_window(production_stats.overall.start_date, date_window, timezone),
			row_count: production_stats.overall.predictions_count,
			stats: ProductionTrainingStats {
				production: production_stats.overall.stats.as_ref().map(|stats| {
					NumberColumnStats {
						max: stats.max,
						min: stats.min,
						mean: stats.mean,
						p25: stats.p25,
						p50: stats.p50,
						p75: stats.p75,
						std: stats.std,
					}
				}),
				training: NumberColumnStats {
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

	let intervals = production_stats
		.intervals
		.into_iter()
		.map(|interval| NumberColumnStatsInterval {
			label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
			stats: interval.stats.map(|c| NumberColumnStats {
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

	NumberColumnStatsViewModel {
		alert: production_stats.overall.alert,
		name: production_stats.overall.column_name,
		date_window,
		date_window_interval,
		intervals,
		overall,
	}
}

fn build_production_column_enum_stats(
	production_stats: types::EnumProductionStatsSingleColumnResponse,
	train_column_stats: tangram_core::types::EnumColumnStats,
	train_row_count: u64,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> EnumColumnStatsViewModel {
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
					OverallEnumColumnStatsEntry {
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

	let overall = EnumColumnStatsOverall {
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
		.map(|interval| EnumColumnStatsInterval {
			label: format_date_window_interval(interval.start_date, date_window_interval, timezone),
			histogram: interval.histogram,
		})
		.collect();

	EnumColumnStatsViewModel {
		alert: production_stats.overall.alert,
		name: production_stats.overall.column_name,
		date_window,
		date_window_interval,
		intervals,
		overall,
	}
}

fn build_production_column_text_stats(
	production_stats: types::TextProductionStatsSingleColumnResponse,
	_train_column_stats: tangram_core::types::TextColumnStats,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> TextColumnStatsViewModel {
	let overall = TextColumnStatsOverall {
		absent_count: production_stats.overall.absent_count,
		invalid_count: production_stats.overall.invalid_count,
		label: format_date_window(production_stats.overall.start_date, date_window, timezone),
		row_count: production_stats.overall.predictions_count,
		token_histogram: production_stats.overall.token_histogram,
	};

	TextColumnStatsViewModel {
		alert: production_stats.overall.alert,
		name: production_stats.overall.column_name,
		date_window,
		date_window_interval,
		overall,
	}
}
