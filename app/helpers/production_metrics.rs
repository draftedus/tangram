use crate::types::{DateWindow, DateWindowInterval};
use anyhow::Result;
use chrono::prelude::*;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use sqlx::prelude::*;
use tangram_core::metrics::RunningMetric;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetrics {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionMetrics,
	pub intervals: Vec<ProductionMetrics>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: Option<PredictionMetrics>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum PredictionMetrics {
	Regression(RegressionPredictionMetrics),
	Classification(ClassificationPredictionMetrics),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictionMetrics {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictionMetrics {
	pub class_metrics: Vec<ClassificationPredictionClassMetrics>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictionClassMetrics {
	pub class_name: String,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

pub async fn get_production_metrics(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: &tangram_core::types::Model,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<ProductionMetrics> {
	// compute the start date given the date window.
	// * for today, use the start of this utc day.
	// * for this month, use the start of this utc month.
	// * for this year, use the start of this utc year.
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
	let rows = sqlx::query(
		"
			select
				data,
				hour
			from production_metrics
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
	let mut intervals: Vec<ProductionMetrics> = (0..n_intervals.to_u32().unwrap())
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
			ProductionMetrics::new(&model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// merge each hourly production metrics
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

	let intervals: Vec<ProductionMetrics> = intervals
		.into_iter()
		.map(|metrics| metrics.finalize())
		.collect();

	let response = ProductionMetrics {
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
