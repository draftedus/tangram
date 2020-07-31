use crate::types;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

pub fn format_date_window(
	date: DateTime<Utc>,
	date_window: types::DateWindow,
	timezone: Tz,
) -> String {
	let date = date.with_timezone(&timezone);
	match date_window {
		types::DateWindow::Today => format_day(date),
		types::DateWindow::ThisMonth => format_month(date),
		types::DateWindow::ThisYear => format_year(date),
	}
}

pub fn format_date_window_interval(
	date: DateTime<Utc>,
	date_window_interval: types::DateWindowInterval,
	timezone: Tz,
) -> String {
	let date = date.with_timezone(&timezone);
	match date_window_interval {
		types::DateWindowInterval::Hourly => format_hour(date),
		types::DateWindowInterval::Daily => format_day_of_month(date),
		types::DateWindowInterval::Monthly => format_month(date),
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
