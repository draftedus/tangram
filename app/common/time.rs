use crate::common::date_window::{DateWindow, DateWindowInterval};
use chrono::prelude::*;
use chrono_tz::Tz;

pub fn format_date_window(date: DateTime<Utc>, date_window: DateWindow, timezone: Tz) -> String {
	let date = date.with_timezone(&timezone);
	match date_window {
		DateWindow::Today => format_day(date),
		DateWindow::ThisMonth => format_month(date),
		DateWindow::ThisYear => format_year(date),
	}
}

pub fn format_date_window_interval(
	date: DateTime<Utc>,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> String {
	let date = date.with_timezone(&timezone);
	match date_window_interval {
		DateWindowInterval::Hourly => format_hour(date),
		DateWindowInterval::Daily => format_day_of_month(date),
		DateWindowInterval::Monthly => format_month(date),
	}
}

pub fn format_hour(date: DateTime<Tz>) -> String {
	date.format("%-l%P").to_string()
}

pub fn format_day(date: DateTime<Tz>) -> String {
	date.format("%a %b %d %Y").to_string()
}

pub fn format_day_of_month(date: DateTime<Tz>) -> String {
	date.format("%b %d").to_string()
}

pub fn format_month(date: DateTime<Tz>) -> String {
	date.format("%b %Y").to_string()
}

pub fn format_year(date: DateTime<Tz>) -> String {
	date.format("%Y").to_string()
}
