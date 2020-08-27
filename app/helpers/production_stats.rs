use crate::types::{DateWindow, DateWindowInterval};
use anyhow::Result;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use tangram_core::metrics::RunningMetric;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStats {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionStats,
	pub intervals: Vec<ProductionStats>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStatsInterval {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_stats: Vec<ProductionColumnStats>,
	pub prediction_stats: ProductionPredictionStats,
}

pub async fn get_production_stats(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: &tangram_core::types::Model,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<ProductionStats> {
	// compute the start date given the date window
	// * for today, use the start of this utc day
	// * for this month, use the start of this utc month
	// * for this year, use the start of this utc year
	let now: DateTime<Tz> = Utc::now().with_timezone(&timezone);
	let start_date = match date_window {
		DateWindow::Today => timezone
			.ymd(now.year(), now.month(), now.day())
			.and_hms(0, 0, 0),
		DateWindow::ThisMonth => timezone.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0),
		DateWindow::ThisYear => timezone.ymd(now.year(), 1, 1).and_hms(0, 0, 0),
	};
	let end_date = match date_window {
		DateWindow::Today => start_date + Duration::days(1),
		DateWindow::ThisMonth => {
			start_date + Duration::days(n_days_in_month(start_date.year(), start_date.month()))
		}
		DateWindow::ThisYear => timezone.ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0),
	};
	let start_date = &start_date.with_timezone(&Utc);
	// retrieve the production stats for the date window
	let rows = sqlx::query(
		"
			select
				data,
				hour
			from production_stats
			where
				model_id = ?1 and
				hour >= ?2
			order by hour
		",
	)
	.bind(&model.id().to_string())
	.bind(&start_date.timestamp())
	.fetch_all(&mut *db)
	.await?;
	// compute the number of intervals
	// * for today, use 24
	// * for this month, use the number of days in this month
	// * for this year, use 12
	let n_intervals: usize = match date_window_interval {
		DateWindowInterval::Hourly => 24,
		DateWindowInterval::Daily => n_days_in_month(start_date.year(), start_date.month())
			.to_usize()
			.unwrap(),
		DateWindowInterval::Monthly => 12,
	};
	// initialize the intervals with start and end dates
	let mut intervals: Vec<ProductionStats> = (0..n_intervals.to_u32().unwrap())
		.map(|i| {
			// determine the start and end dates for the interval
			let start = match date_window_interval {
				DateWindowInterval::Hourly => start_date.with_hour(i).unwrap(),
				DateWindowInterval::Daily => start_date.with_day0(i).unwrap(),
				DateWindowInterval::Monthly => start_date.with_month0(i).unwrap(),
			};
			let end = match date_window_interval {
				DateWindowInterval::Hourly => start + Duration::hours(1),
				DateWindowInterval::Daily => start + Duration::days(1),
				DateWindowInterval::Monthly => {
					start + Duration::days(n_days_in_month(start.year(), start.month()))
				}
			};
			ProductionStats::new(&model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// merge each hourly production stats
	// entry into its corresponding interval
	for row in rows {
		let data: String = row.get(0);
		let data: Vec<u8> = base64::decode(data)?;
		let hour: i64 = row.get(1);
		let hour: DateTime<Utc> = Utc.datetime_from_str(hour.to_string().as_str(), "%s")?;
		let hour: DateTime<Tz> = hour.with_timezone(&timezone);
		let interval = match date_window_interval {
			DateWindowInterval::Hourly => {
				let hour = hour.hour().to_usize().unwrap();
				intervals.get_mut(hour).unwrap()
			}
			DateWindowInterval::Daily => {
				let day = hour.day0().to_usize().unwrap();
				intervals.get_mut(day).unwrap()
			}
			DateWindowInterval::Monthly => {
				let month = hour.month0().to_usize().unwrap();
				intervals.get_mut(month).unwrap()
			}
		};
		let hourly_production_stats = serde_json::from_slice(&data)?;
		interval.merge(hourly_production_stats);
	}
	// compute the overall production stats by merging all the intervals together
	let overall = intervals
		.iter()
		.fold(
			ProductionStatsInterval::new(
				&model,
				start_date.with_timezone(&Utc),
				end_date.with_timezone(&Utc),
			),
			|mut stats, next| {
				stats.merge(next.clone());
				stats
			},
		)
		.finalize();
	// finalize the intervals
	let intervals: Vec<ProductionStatsInterval> = intervals
		.into_iter()
		.map(|stats| stats.finalize())
		.collect();
	// assemble the response
	let response = ProductionStats {
		date_window,
		date_window_interval,
		overall,
		intervals,
	};
	Ok(response)
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum ProductionColumnStats {
	Unknown(UnknownProductionColumnStats),
	Text(TextProductionColumnStats),
	Number(NumberProductionColumnStats),
	Enum(EnumProductionColumnStats),
}

impl ProductionColumnStats {
	pub fn column_name(&self) -> &str {
		match &self {
			Self::Unknown(c) => c.column_name.as_str(),
			Self::Text(c) => c.column_name.as_str(),
			Self::Number(c) => c.column_name.as_str(),
			Self::Enum(c) => c.column_name.as_str(),
		}
	}
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnknownProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberStats {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub token_histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum ProductionPredictionStats {
	Regression(RegressionProductionPredictionStats),
	Classification(ClassificationProductionPredictionStats),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionProductionPredictionStats {
	pub stats: Option<NumberStats>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationProductionPredictionStats {
	pub histogram: Vec<(String, u64)>,
}

pub enum ProductionStatsSingleColumnResponse {
	Number(NumberProductionStatsSingleColumnResponse),
	Enum(EnumProductionStatsSingleColumnResponse),
	Text(TextProductionStatsSingleColumnResponse),
}

pub struct NumberProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: NumberProductionStatsSingleColumn,
	pub intervals: Vec<NumberProductionStatsSingleColumn>,
}

pub struct NumberProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub alert: Option<String>,
}

pub struct EnumProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: EnumProductionStatsSingleColumn,
	pub intervals: Vec<EnumProductionStatsSingleColumn>,
}

pub struct EnumProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
	pub alert: Option<String>,
}

pub struct TextProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: TextProductionStatsSingleColumn,
}

pub struct TextProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub token_histogram: Vec<(String, u64)>,
	pub alert: Option<String>,
}

pub async fn get_production_column_stats(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: &tangram_core::types::Model,
	column_name: &str,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<ProductionStatsSingleColumnResponse> {
	let production_stats =
		get_production_stats(db, model, date_window, date_window_interval, timezone).await?;

	let production_column_stats = production_stats
		.overall
		.column_stats
		.into_iter()
		.find(|c| c.column_name() == column_name)
		.unwrap();

	let production_column_stats: ProductionStatsSingleColumnResponse = match production_column_stats
	{
		ProductionColumnStats::Number(c) => {
			let overall = NumberProductionStatsSingleColumn {
				start_date: production_stats.overall.start_date,
				end_date: production_stats.overall.end_date,
				predictions_count: production_stats.overall.predictions_count,
				column_name: c.column_name.to_owned(),
				absent_count: c.absent_count,
				invalid_count: c.invalid_count,
				stats: c.stats,
				alert: c.alert,
			};

			let intervals = production_stats
				.intervals
				.into_iter()
				.map(|interval| {
					let column_stats = interval
						.column_stats
						.into_iter()
						.find(|c| c.column_name() == column_name)
						.unwrap();
					match column_stats {
						ProductionColumnStats::Number(c) => NumberProductionStatsSingleColumn {
							start_date: interval.start_date,
							end_date: interval.end_date,
							predictions_count: interval.predictions_count,
							column_name: c.column_name.to_owned(),
							absent_count: c.absent_count,
							invalid_count: c.invalid_count,
							stats: c.stats,
							alert: c.alert,
						},
						_ => unreachable!(),
					}
				})
				.collect();

			ProductionStatsSingleColumnResponse::Number(NumberProductionStatsSingleColumnResponse {
				date_window: production_stats.date_window,
				date_window_interval: production_stats.date_window_interval,
				overall,
				intervals,
			})
		}
		ProductionColumnStats::Enum(c) => {
			let overall = EnumProductionStatsSingleColumn {
				start_date: production_stats.overall.start_date,
				end_date: production_stats.overall.end_date,
				predictions_count: production_stats.overall.predictions_count,
				column_name: c.column_name.to_owned(),
				absent_count: c.absent_count,
				invalid_count: c.invalid_count,
				histogram: c.histogram,
				invalid_histogram: c.invalid_histogram,
				alert: c.alert,
			};

			let intervals = production_stats
				.intervals
				.into_iter()
				.map(|interval| {
					let column_stats = interval
						.column_stats
						.into_iter()
						.find(|c| c.column_name() == column_name)
						.unwrap();
					match column_stats {
						ProductionColumnStats::Enum(c) => EnumProductionStatsSingleColumn {
							start_date: interval.start_date,
							end_date: interval.end_date,
							predictions_count: interval.predictions_count,
							column_name: c.column_name.to_owned(),
							absent_count: c.absent_count,
							invalid_count: c.invalid_count,
							histogram: c.histogram,
							invalid_histogram: c.invalid_histogram,
							alert: c.alert,
						},
						_ => unreachable!(),
					}
				})
				.collect();
			ProductionStatsSingleColumnResponse::Enum(EnumProductionStatsSingleColumnResponse {
				date_window: production_stats.date_window,
				date_window_interval: production_stats.date_window_interval,
				overall,
				intervals,
			})
		}
		ProductionColumnStats::Text(c) => {
			let overall = TextProductionStatsSingleColumn {
				start_date: production_stats.overall.start_date,
				end_date: production_stats.overall.end_date,
				predictions_count: production_stats.overall.predictions_count,
				column_name: c.column_name.to_owned(),
				absent_count: c.absent_count,
				invalid_count: c.invalid_count,
				token_histogram: c.token_histogram,
				alert: c.alert,
			};

			ProductionStatsSingleColumnResponse::Text(TextProductionStatsSingleColumnResponse {
				date_window: production_stats.date_window,
				date_window_interval: production_stats.date_window_interval,
				overall,
			})
		}
		_ => unreachable!(),
	};

	Ok(production_column_stats)
}

fn n_days_in_month(year: i32, month: u32) -> i64 {
	let (end_year, end_month) = if month == 12 {
		(year + 1, 1)
	} else {
		(year, month + 1)
	};
	let start = NaiveDate::from_ymd(year, month, 1);
	let end = NaiveDate::from_ymd(end_year, end_month, 1);
	(end - start).num_days()
}
