use crate::date_window::{self, DateWindow, DateWindowInterval};
use tangram_deps::{chrono::prelude::*, chrono_tz::Tz};

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

pub fn interval_chart_title(date_window_interval: DateWindowInterval, title: String) -> String {
	match date_window_interval {
		DateWindowInterval::Hourly => format!("Hourly {}", title),
		DateWindowInterval::Daily => format!("Daily {}", title),
		DateWindowInterval::Monthly => format!("Monthly {}", title),
	}
}

pub fn overall_chart_title(date_window: DateWindow, title: String) -> String {
	match date_window {
		DateWindow::Today => format!("Today's {}", title),
		DateWindow::ThisMonth => format!("This Month's {}", title),
		DateWindow::ThisYear => format!("This Year's {}", title),
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
