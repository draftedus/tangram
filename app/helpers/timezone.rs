use super::cookies::parse_cookies;
use chrono_tz::{Tz, UTC};
use hyper::{header, Body, Request};

pub fn get_timezone(request: &Request<Body>) -> Tz {
	request
		.headers()
		.get(header::COOKIE)
		.and_then(|cookie_header_value| cookie_header_value.to_str().ok())
		.and_then(|cookie_header_value| parse_cookies(cookie_header_value).ok())
		.and_then(|cookies| cookies.get("tangram-timezone").cloned())
		.and_then(|timezone_str| timezone_str.parse().ok())
		.unwrap_or(UTC)
}
