use crate::{production_metrics::ProductionMetrics, types};
use anyhow::Result;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use tangram_core::metrics::RunningMetric;

pub async fn get_production_metrics(
	db: &deadpool_postgres::Transaction<'_>,
	model: &tangram_core::types::Model,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> Result<types::ProductionMetricsResponse> {
	// compute the start date given the date window.
	// * for today, use the start of this utc day.
	// * for this month, use the start of this utc month.
	// * for this year, use the start of this utc year.
	let now: DateTime<Tz> = Utc::now().with_timezone(&timezone);
	let start_date = match date_window {
		types::DateWindow::Today => timezone
			.ymd(now.year(), now.month(), now.day())
			.and_hms(0, 0, 0),
		types::DateWindow::ThisMonth => timezone.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0),
		types::DateWindow::ThisYear => timezone.ymd(now.year(), 1, 1).and_hms(0, 0, 0),
	};
	let end_date = match date_window {
		types::DateWindow::Today => start_date + Duration::days(1),
		types::DateWindow::ThisMonth => {
			start_date + Duration::days(n_days_in_month(start_date.year(), start_date.month()))
		}
		types::DateWindow::ThisYear => timezone.ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0),
	};
	let rows = db
		.query(
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
			&[&model.id(), &start_date.with_timezone(&Utc)],
		)
		.await?;

	// compute the number of intervals
	// * for today, use 24
	// * for this month, use the number of days in this month
	// * for this year, use 12
	let n_intervals: usize = match date_window_interval {
		types::DateWindowInterval::Hourly => 24,
		types::DateWindowInterval::Daily => n_days_in_month(start_date.year(), start_date.month())
			.to_usize()
			.unwrap(),
		types::DateWindowInterval::Monthly => 12,
	};
	let mut intervals: Vec<ProductionMetrics> = (0..n_intervals.to_u32().unwrap())
		.map(|i| {
			// determine the start and end dates for the interval
			let start = match date_window_interval {
				types::DateWindowInterval::Hourly => start_date.with_hour(i).unwrap(),
				types::DateWindowInterval::Daily => start_date.with_day0(i).unwrap(),
				types::DateWindowInterval::Monthly => start_date.with_month0(i).unwrap(),
			};
			let end = match date_window_interval {
				types::DateWindowInterval::Hourly => start + Duration::hours(1),
				types::DateWindowInterval::Daily => start + Duration::days(1),
				types::DateWindowInterval::Monthly => {
					start + Duration::days(n_days_in_month(start.year(), start.month()))
				}
			};
			ProductionMetrics::new(&model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// merge each hourly production metrics
	// entry into its corresponding interval
	for row in rows {
		let data: Vec<u8> = row.get(0);
		let hour: DateTime<Utc> = row.get(1);
		let hour: DateTime<Tz> = hour.with_timezone(&timezone);
		let interval = match date_window_interval {
			types::DateWindowInterval::Hourly => {
				let hour = hour.hour().to_usize().unwrap();
				intervals.get_mut(hour).unwrap()
			}
			types::DateWindowInterval::Daily => {
				let day = hour.day0().to_usize().unwrap();
				intervals.get_mut(day).unwrap()
			}
			types::DateWindowInterval::Monthly => {
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

	let intervals: Vec<types::ProductionMetrics> = intervals
		.into_iter()
		.map(|metrics| metrics.finalize())
		.collect();

	let response = types::ProductionMetricsResponse {
		date_window,
		date_window_interval,
		overall,
		intervals,
	};

	Ok(response)
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
