use crate::{production_stats::ProductionStats, types};
use anyhow::Result;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use num_traits::ToPrimitive;
use tangram_core::metrics::RunningMetric;
use tokio_postgres as postgres;

pub async fn get_production_column_stats(
	db: &postgres::Transaction<'_>,
	model: &tangram_core::types::Model,
	column_name: &str,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> Result<types::ProductionStatsSingleColumnResponse> {
	let production_stats =
		get_production_stats(db, model, date_window, date_window_interval, timezone).await?;

	let production_column_stats = production_stats
		.overall
		.column_stats
		.into_iter()
		.find(|c| c.column_name() == column_name)
		.unwrap();

	let production_column_stats: types::ProductionStatsSingleColumnResponse =
		match production_column_stats {
			types::ProductionColumnStats::Number(c) => {
				let overall = types::NumberProductionStatsSingleColumn {
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
							types::ProductionColumnStats::Number(c) => {
								types::NumberProductionStatsSingleColumn {
									start_date: interval.start_date,
									end_date: interval.end_date,
									predictions_count: interval.predictions_count,
									column_name: c.column_name.to_owned(),
									absent_count: c.absent_count,
									invalid_count: c.invalid_count,
									stats: c.stats,
									alert: c.alert,
								}
							}
							_ => unreachable!(),
						}
					})
					.collect();

				types::ProductionStatsSingleColumnResponse::Number(
					types::NumberProductionStatsSingleColumnResponse {
						date_window: production_stats.date_window,
						date_window_interval: production_stats.date_window_interval,
						overall,
						intervals,
					},
				)
			}
			types::ProductionColumnStats::Enum(c) => {
				let overall = types::EnumProductionStatsSingleColumn {
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
							types::ProductionColumnStats::Enum(c) => {
								types::EnumProductionStatsSingleColumn {
									start_date: interval.start_date,
									end_date: interval.end_date,
									predictions_count: interval.predictions_count,
									column_name: c.column_name.to_owned(),
									absent_count: c.absent_count,
									invalid_count: c.invalid_count,
									histogram: c.histogram,
									invalid_histogram: c.invalid_histogram,
									alert: c.alert,
								}
							}
							_ => unreachable!(),
						}
					})
					.collect();
				types::ProductionStatsSingleColumnResponse::Enum(
					types::EnumProductionStatsSingleColumnResponse {
						date_window: production_stats.date_window,
						date_window_interval: production_stats.date_window_interval,
						overall,
						intervals,
					},
				)
			}
			types::ProductionColumnStats::Text(c) => {
				let overall = types::TextProductionStatsSingleColumn {
					start_date: production_stats.overall.start_date,
					end_date: production_stats.overall.end_date,
					predictions_count: production_stats.overall.predictions_count,
					column_name: c.column_name.to_owned(),
					absent_count: c.absent_count,
					invalid_count: c.invalid_count,
					token_histogram: c.token_histogram,
					alert: c.alert,
				};

				types::ProductionStatsSingleColumnResponse::Text(
					types::TextProductionStatsSingleColumnResponse {
						date_window: production_stats.date_window,
						date_window_interval: production_stats.date_window_interval,
						overall,
					},
				)
			}
			_ => unreachable!(),
		};

	Ok(production_column_stats)
}

pub async fn get_production_stats(
	db: &postgres::Transaction<'_>,
	model: &tangram_core::types::Model,
	date_window: types::DateWindow,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> Result<types::ProductionStatsResponse> {
	// compute the start date given the date window
	// * for today, use the start of this utc day
	// * for this month, use the start of this utc month
	// * for this year, use the start of this utc year
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
	// retrieve the production stats for the date window
	let rows = db
		.query(
			"
				select
					data,
					hour
				from production_stats
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
	// initialize the intervals with start and end dates
	let mut intervals: Vec<ProductionStats> = (0..n_intervals.to_u32().unwrap())
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
			ProductionStats::new(&model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// merge each hourly production stats
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
	let intervals: Vec<types::ProductionStats> = intervals
		.into_iter()
		.map(|stats| stats.finalize())
		.collect();
	// assemble the response
	let response = types::ProductionStatsResponse {
		date_window,
		date_window_interval,
		overall,
		intervals,
	};
	Ok(response)
}

// pub async fn get(
// 	request: Request<Body>,
// 	context: Arc<Context>,
// 	model_id: &str,
// 	search_params: Option<BTreeMap<String, String>>,
// ) -> Result<Response<Body>> {
// 	// parse the date window search param
// 	let date_window = search_params
// 		.as_ref()
// 		.and_then(|query| query.get("date_window"));
// 	let date_window = if let Some(date_window) = date_window {
// 		date_window.as_str()
// 	} else {
// 		return Err(Error::BadRequest.into());
// 	};
// 	let date_window = match date_window {
// 		"today" => types::DateWindow::Today,
// 		"this_month" => types::DateWindow::ThisMonth,
// 		"this_year" => types::DateWindow::ThisYear,
// 		_ => return Err(Error::BadRequest.into()),
// 	};
// 	// choose the interval to use for the date window
// 	let date_window_interval = match date_window {
// 		types::DateWindow::Today => types::DateWindowInterval::Hourly,
// 		types::DateWindow::ThisMonth => types::DateWindowInterval::Daily,
// 		types::DateWindow::ThisYear => types::DateWindowInterval::Monthly,
// 	};
// 	// get the timezone
// 	let timezone = request
// 		.headers()
// 		.get(header::COOKIE)
// 		.and_then(|cookie_header_value| cookie_header_value.to_str().ok())
// 		.and_then(|cookie_header_value| cookies::parse(cookie_header_value).ok())
// 		.and_then(|cookies| cookies.get("tangram-timezone").cloned())
// 		.and_then(|timezone_str| timezone_str.parse().ok())
// 		.unwrap_or(UTC);

// 	let mut db = if let Ok(db) = context.database_pool.get().await {
// 		db
// 	} else {
// 		return Err(Error::ServiceUnavailable.into());
// 	};
// 	let db = db.transaction().await?;
// 	let user = if let Ok(user) = authorize_user(&request, &db).await? {
// 		user
// 	} else {
// 		return Err(Error::Unauthorized.into());
// 	};
// 	let model_id: Id = if let Ok(model_id) = model_id.parse() {
// 		model_id
// 	} else {
// 		return Err(Error::NotFound.into());
// 	};
// 	if !authorize_user_for_model(&db, &user, model_id).await? {
// 		return Err(Error::NotFound.into());
// 	}
// 	let model = if let Ok(model) = get_model(&db, model_id).await {
// 		model
// 	} else {
// 		return Err(Error::NotFound.into());
// 	};

// 	let production_stats =
// 		get_production_stats(&db, &model, date_window, date_window_interval, timezone).await?;
// 	let response = serde_json::to_vec(&production_stats)?;
// 	Ok(Response::builder()
// 		.status(StatusCode::OK)
// 		.header(header::CONTENT_TYPE, "application/json")
// 		.body(Body::from(response))?)
// }

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
