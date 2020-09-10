use crate::{
	common::date_window::{DateWindow, DateWindowInterval},
	production_stats::{ProductionStats, ProductionStatsOutput},
};
use anyhow::Result;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use tangram_core::metrics::Metric;

#[derive(Debug)]
pub struct GetProductionStatsOutput {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionStatsOutput,
	pub intervals: Vec<ProductionStatsOutput>,
}

pub async fn get_production_stats(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: &tangram_core::types::Model,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<GetProductionStatsOutput> {
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
			ProductionStats::new(
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
	let intervals: Vec<ProductionStatsOutput> = intervals
		.into_iter()
		.map(|stats| stats.finalize())
		.collect();
	Ok(GetProductionStatsOutput {
		date_window,
		date_window_interval,
		overall,
		intervals,
	})
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
