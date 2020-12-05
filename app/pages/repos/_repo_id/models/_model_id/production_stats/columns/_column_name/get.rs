use super::page::{
	render, EnumColumnProps, EnumOverallHistogramEntry, Inner, IntervalBoxChartDataPoint,
	IntervalBoxChartDataPointStats, NumberColumnProps, NumberTrainingProductionComparison,
	OverallBoxChartData, OverallBoxChartDataStats, Props, TextColumnProps,
};
use std::collections::BTreeMap;
use tangram_app_common::{
	date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	production_stats::ProductionColumnStatsOutput,
	production_stats::{get_production_stats, GetProductionStatsOutput},
	time::format_date_window_interval,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::{chrono_tz::Tz, http, hyper, num_traits::ToPrimitive};
use tangram_util::{client, error::Result, id::Id, zip};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	column_name: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<http::Response<hyper::Body>> {
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
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
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model = get_model(&mut db, model_id).await?;
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let get_production_stats_output = get_production_stats(
		&mut db,
		&model,
		date_window.clone(),
		date_window_interval.clone(),
		timezone,
	)
	.await?;
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
				date_window.clone(),
				date_window_interval,
				timezone,
			))
		}
		tangram_core::model::ColumnStats::Enum(train_column_stats) => Inner::Enum(enum_props(
			get_production_stats_output,
			train_column_stats,
			train_row_count,
			date_window.clone(),
			date_window_interval,
			timezone,
		)),
		tangram_core::model::ColumnStats::Text(train_column_stats) => Inner::Text(text_props(
			get_production_stats_output,
			train_column_stats,
			date_window.clone(),
			date_window_interval,
			timezone,
		)),
		_ => return Ok(bad_request()),
	};
	db.commit().await?;
	let props = Props {
		date_window,
		column_name: column_name.to_owned(),
		id: model_id.to_string(),
		inner,
		model_layout_info,
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

fn number_props(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: &tangram_core::model::NumberColumnStats,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> NumberColumnProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Number(overall) => overall,
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
				ProductionColumnStatsOutput::Number(production_column_stats) => {
					production_column_stats
				}
				_ => unreachable!(),
			};
			IntervalBoxChartDataPoint {
				label: format_date_window_interval(
					interval.start_date,
					&date_window_interval,
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
	NumberColumnProps {
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
) -> EnumColumnProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Enum(overall) => overall,
		_ => unreachable!(),
	};
	let production_row_count = get_production_stats_output.overall.row_count;
	let overall_chart_data = zip!(
		overall.histogram.iter(),
		train_column_stats.histogram.iter(),
	)
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
	EnumColumnProps {
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
) -> TextColumnProps {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Text(overall) => overall,
		_ => unreachable!(),
	};
	let overall_token_histogram = overall
		.token_histogram
		.iter()
		.map(|(token, count)| (token.to_string(), *count))
		.collect();
	TextColumnProps {
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
