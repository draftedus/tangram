use anyhow::{Context, Result};
use clap::Clap;
use colored::*;
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
};
use url::Url;

#[cfg(feature = "train")]
mod progress;

#[derive(Clap)]
#[clap(about = "Train and deploy a machine learning model in minutes.")]
enum Options {
	#[cfg(feature = "train")]
	#[clap(name = "train")]
	Train(TrainOptions),
	#[cfg(feature = "app")]
	#[clap(name = "app")]
	App(AppOptions),
}

#[cfg(feature = "train")]
#[derive(Clap)]
#[clap(about = "train a model")]
#[clap(long_about = "train a model from a csv file with a specified target column")]
struct TrainOptions {
	#[clap(short, long, about = "the path to your .csv file")]
	file: PathBuf,
	#[clap(short, long, about = "the name of the column to predict")]
	target: String,
	#[clap(short, long, about = "the path to a config file")]
	config: Option<PathBuf>,
	#[clap(short, long, about = "the path to write the output to")]
	output: Option<PathBuf>,
	#[clap(long = "no-progress", parse(from_flag = std::ops::Not::not))]
	progress: bool,
}

#[cfg(feature = "app")]
#[derive(Clap)]
#[clap(about = "run the app")]
#[clap(long_about = "run the reporting and monitoring app")]
struct AppOptions {
	#[clap(long, env = "AUTH_ENABLED")]
	auth_enabled: bool,
	#[clap(long, env = "COOKIE_DOMAIN")]
	cookie_domain: Option<String>,
	#[clap(long, env = "DATABASE_URL")]
	database_url: Option<Url>,
	#[clap(long, default_value = "0.0.0.0")]
	host: std::net::IpAddr,
	#[clap(long, env = "PORT", default_value = "8080")]
	port: u16,
	#[clap(long, env = "SENDGRID_API_TOKEN")]
	sendgrid_api_token: Option<String>,
	#[clap(hidden = true, long, env = "STRIPE_PUBLISHABLE_KEY")]
	stripe_publishable_key: Option<String>,
	#[clap(hidden = true, long, env = "STRIPE_SECRET_KEY")]
	stripe_secret_key: Option<String>,
	#[clap(hidden = true, long, env = "URL")]
	url: Option<Url>,
}

fn main() {
	let options = Options::parse();
	let result = match options {
		#[cfg(feature = "train")]
		Options::Train(options) => cli_train(options),
		#[cfg(feature = "app")]
		Options::App(options) => cli_app(options),
	};
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		error
			.chain()
			.skip(1)
			.for_each(|cause| eprintln!("  {} {}", "->".red().bold(), cause));
		std::process::exit(1);
	}
}

#[cfg(feature = "train")]
fn cli_train(options: TrainOptions) -> Result<()> {
	let mut progress_view = if options.progress {
		progress::ProgressView::new().ok()
	} else {
		None
	};
	let mut update_progress = |progress| match progress_view.as_mut() {
		Some(progress_manager) => progress_manager.update(progress),
		None => {}
	};
	let model = tangram::train(
		tangram::id::Id::new(),
		&options.file,
		&options.target,
		options.config.as_deref(),
		&mut update_progress,
	)?;
	drop(progress_view);

	let output_path = match options.output.as_deref() {
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = options.file.file_stem().unwrap().to_str().unwrap();
			Cow::Owned(available_path(&dir, csv_file_name, "tangram"))
		}
		Some(path) => Cow::Borrowed(path),
	};

	model
		.to_file(&output_path)
		.context("failed to write model to file")?;

	eprintln!("Your model was written to {}.", output_path.display());
	eprintln!(
		"For help making predictions in your code, read the docs at https://www.tangramhq.com/docs."
	);
	eprintln!(
		"To learn more about how your model works and set up production monitoring, upload your .tangram file at https://app.tangramhq.com/ or your on-prem Tangram deployment."
	);

	Ok(())
}

#[cfg(feature = "app")]
fn cli_app(options: AppOptions) -> Result<()> {
	tangram_app::run(tangram_app::AppOptions {
		auth_enabled: options.auth_enabled,
		cookie_domain: options.cookie_domain,
		database_url: options.database_url,
		host: options.host,
		port: options.port,
		sendgrid_api_token: options.sendgrid_api_token,
		stripe_publishable_key: options.stripe_publishable_key,
		stripe_secret_key: options.stripe_secret_key,
		url: options.url,
	})
}

fn available_path(base: &Path, name: &str, extension: &str) -> PathBuf {
	let mut i = 0;
	loop {
		let mut path = PathBuf::from(base);
		let mut filename = String::new();
		filename.push_str(name);
		if i > 0 {
			filename.push(' ');
			filename.push_str(&i.to_string());
		}
		filename.push('.');
		filename.push_str(extension);
		path.push(filename);
		if !path.exists() {
			return path;
		}
		i += 1;
	}
}
