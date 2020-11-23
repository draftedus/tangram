//! This module contains the main entrypoint to the tangram cli.

use clap::Clap;
use colored::Colorize;
use std::path::PathBuf;
use url::Url;

#[cfg(feature = "app")]
mod app;
#[cfg(feature = "train")]
mod train;

#[derive(Clap)]
#[clap(
	about = "Train and deploy a machine learning model in minutes.",
	setting = clap::AppSettings::DisableHelpSubcommand,
)]
enum Args {
	#[cfg(feature = "train")]
	#[clap(name = "train")]
	Train(Box<TrainArgs>),
	#[cfg(feature = "app")]
	#[clap(name = "app")]
	App(Box<AppArgs>),
}

#[cfg(feature = "train")]
#[derive(Clap)]
#[clap(about = "train a model")]
#[clap(long_about = "train a model from a csv file")]
struct TrainArgs {
	#[clap(short, long, about = "the path to your .csv file", conflicts_with_all=&["file-train", "file-test"])]
	file: Option<PathBuf>,
	#[clap(
		long,
		about = "the path to your .csv file used for training",
		requires = "file-test"
	)]
	file_train: Option<PathBuf>,
	#[clap(
		long,
		about = "the path to your .csv file used for testing",
		requires = "file-train"
	)]
	file_test: Option<PathBuf>,
	#[clap(short, long, about = "the name of the column to predict")]
	target: String,
	#[clap(short, long, about = "the path to a config file")]
	config: Option<PathBuf>,
	#[clap(short, long, about = "the path to write the .tangram file to")]
	output: Option<PathBuf>,
	#[clap(long = "no-progress", about = "disable the cli progress view", parse(from_flag = std::ops::Not::not))]
	progress: bool,
}

#[cfg(feature = "app")]
#[derive(Clap)]
#[clap(about = "run the app")]
#[clap(long_about = "run the reporting and monitoring web app")]
struct AppArgs {
	#[clap(long = "auth", env = "AUTH", takes_value = false)]
	auth_enabled: bool,
	#[clap(long, env = "COOKIE_DOMAIN")]
	cookie_domain: Option<String>,
	#[clap(long, env = "DATABASE_URL")]
	database_url: Option<Url>,
	#[clap(long, env = "DATABASE_MAX_CONNECTIONS")]
	database_max_connections: Option<u32>,
	#[clap(long, default_value = "0.0.0.0")]
	host: std::net::IpAddr,
	#[clap(long, env = "PORT", default_value = "8080")]
	port: u16,
	#[clap(long, env = "SENDGRID_API_TOKEN")]
	sendgrid_api_token: Option<String>,
	#[clap(long, env = "LICENSE")]
	license: Option<PathBuf>,
	#[clap(hidden = true, long, env = "STRIPE_PUBLISHABLE_KEY")]
	stripe_publishable_key: Option<String>,
	#[clap(hidden = true, long, env = "STRIPE_SECRET_KEY")]
	stripe_secret_key: Option<String>,
	#[clap(hidden = true, long, env = "URL")]
	url: Option<Url>,
}

fn main() {
	let args = Args::parse();
	let result = match args {
		#[cfg(feature = "train")]
		Args::Train(args) => self::train::train(*args),
		#[cfg(feature = "app")]
		Args::App(args) => self::app::app(*args),
	};
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		std::process::exit(1);
	}
}
