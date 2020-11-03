use crate::{
	common::date_window::{DateWindow, DateWindowInterval},
	production_metrics::{ProductionMetrics, ProductionMetricsOutput},
};
use chrono::prelude::*;
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use tangram_metrics::StreamingMetric;
use tangram_util::error::Result;

#[derive(Debug)]
pub struct GetProductionMetricsOutput {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionMetricsOutput,
	pub intervals: Vec<ProductionMetricsOutput>,
}

pub async fn get_production_metrics(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: &tangram_core::model::Model,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<GetProductionMetricsOutput> {
	/*
	Compute the start date given the date window.
	* For today, use the start of this UTC day.
	* For this month, use the start of this UTC month.
	* For this year, use the start of this UTC year.
	*/
	let now: DateTime<Tz> = Utc::now().with_timezone(&timezone);
	let start_date = match date_window {
		DateWindow::Today => timezone
			.ymd(now.year(), now.month(), now.day())
			.and_hms(0, 0, 0),
		DateWindow::ThisMonth => timezone.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0),
		DateWindow::ThisYear => timezone.ymd(now.year(), 1, 1).and_hms(0, 0, 0),
	};
	let end_date = match date_window {
		DateWindow::Today => start_date + chrono::Duration::days(1),
		DateWindow::ThisMonth => {
			start_date
				+ chrono::Duration::days(n_days_in_month(start_date.year(), start_date.month()))
		}
		DateWindow::ThisYear => timezone.ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0),
	};
	let start_date = &start_date.with_timezone(&Utc);
	let rows = sqlx::query(
		"
			select
				data,
				hour
			from production_metrics
			where
				model_id = $1 and
				hour >= $2
			order by hour
		",
	)
	.bind(&model.id().to_string())
	.bind(&start_date.timestamp())
	.fetch_all(&mut *db)
	.await?;
	/*
	 Compute the number of intervals.
	 * For today, use 24.
	 * For this month, use the number of days in this month.
	 * For this year, use 12.
	*/
	let n_intervals: usize = match date_window_interval {
		DateWindowInterval::Hourly => 24,
		DateWindowInterval::Daily => n_days_in_month(start_date.year(), start_date.month())
			.to_usize()
			.unwrap(),
		DateWindowInterval::Monthly => 12,
	};
	let mut intervals: Vec<ProductionMetrics> = (0..n_intervals.to_u32().unwrap())
		.map(|i| {
			// Determine the start and end dates for the interval.
			let start = match date_window_interval {
				DateWindowInterval::Hourly => start_date.with_hour(i).unwrap(),
				DateWindowInterval::Daily => start_date.with_day0(i).unwrap(),
				DateWindowInterval::Monthly => start_date.with_month0(i).unwrap(),
			};
			let end = match date_window_interval {
				DateWindowInterval::Hourly => start + chrono::Duration::hours(1),
				DateWindowInterval::Daily => start + chrono::Duration::days(1),
				DateWindowInterval::Monthly => {
					start + chrono::Duration::days(n_days_in_month(start.year(), start.month()))
				}
			};
			ProductionMetrics::new(&model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// Merge each hourly production metrics entry into its corresponding interval.
	for row in rows {
		let data: String = row.get(0);
		let data: Vec<u8> = base64::decode(data)?;
		let hour: i64 = row.get(1);
		let hour = timezone.timestamp(hour, 0);
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
		let hourly_production_metrics: ProductionMetrics = serde_json::from_slice(&data).unwrap();
		interval.merge(hourly_production_metrics);
	}
	let overall = intervals
		.iter()
		.fold(
			ProductionMetrics::new(
				&model,
				start_date.with_timezone(&Utc),
				end_date.with_timezone(&Utc),
			),
			|mut metrics, next| {
				metrics.merge(next.clone());
				metrics
			},
		)
		.finalize();
	let intervals: Vec<ProductionMetricsOutput> = intervals
		.into_iter()
		.map(|metrics| metrics.finalize())
		.collect();
	Ok(GetProductionMetricsOutput {
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
