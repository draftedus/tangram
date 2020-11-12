pub mod cookies;
pub mod date_window;
pub mod error;
pub mod model;
pub mod monitor_event;
pub mod organizations;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod time;
pub mod timezone;
pub mod topbar;
pub mod user;

pub use base64;
pub use bytes;
pub use chrono;
pub use chrono_tz;
pub use futures;
pub use html;
pub use http;
pub use hyper;
pub use lexical;
pub use multer;
pub use num_traits;
pub use pinwheel;
pub use rand;
pub use reqwest;
pub use serde;
pub use serde_json;
pub use serde_urlencoded;
pub use sqlx;
pub use tokio;
pub use url;

pub struct Options {
	pub auth_enabled: bool,
	pub cookie_domain: Option<String>,
	pub database_url: url::Url,
	pub database_max_connections: Option<u32>,
	pub host: std::net::IpAddr,
	pub port: u16,
	pub sendgrid_api_token: Option<String>,
	pub stripe_publishable_key: Option<String>,
	pub stripe_secret_key: Option<String>,
	pub url: Option<url::Url>,
}

pub struct Context {
	pub options: Options,
	pub pinwheel: pinwheel::Pinwheel,
	pub pool: sqlx::AnyPool,
}
