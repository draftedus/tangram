use tangram_deps::{sqlx, url};

pub mod class_select_field;
pub mod cookies;
pub mod date_window;
pub mod definitions;
pub mod error;
pub mod logo;
pub mod metrics_row;
pub mod model;
pub mod monitor_event;
pub mod organizations;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod time;
pub mod timezone;
pub mod tokens;
pub mod topbar;
pub mod user;

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
	pub pool: sqlx::AnyPool,
}
