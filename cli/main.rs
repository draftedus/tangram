//! This module contains the main entrypoint to the tangram cli.

use anyhow::{anyhow, Context, Result};
use backtrace::Backtrace;
use clap::Clap;
use colored::*;
use once_cell::sync::Lazy;
use progress_view::ProgressView;
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
	sync::Mutex,
};
use url::Url;

mod progress_view;

#[derive(Clap)]
#[clap(
	about = "Train and deploy a machine learning model in minutes.",
	setting = clap::AppSettings::DisableHelpSubcommand,
)]
enum Options {
	#[clap(name = "train")]
	Train(Box<TrainOptions>),
	#[clap(name = "app")]
	App(Box<AppOptions>),
}

#[derive(Clap, Debug)]
#[clap(about = "train a model")]
#[clap(long_about = "train a model from a csv file")]
struct TrainOptions {
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

#[derive(Clap)]
#[clap(about = "run the app")]
#[clap(long_about = "run the reporting and monitoring web app")]
struct AppOptions {
	#[clap(long = "auth", env = "AUTH", takes_value = false)]
	auth_enabled: bool,
	#[clap(long, env = "COOKIE_DOMAIN")]
	cookie_domain: Option<String>,
	#[clap(long, env = "DATABASE_URL")]
	database_url: Option<Url>,
	#[clap(long, env = "DATABASE_POOL_SIZE")]
	database_max_connections: Option<u32>,
	#[clap(long, default_value = "0.0.0.0")]
	host: std::net::IpAddr,
	#[clap(long)]
	model: Option<PathBuf>,
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
		Options::Train(options) => cli_train(*options),
		Options::App(options) => cli_app(*options),
	};
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		for cause in error.chain().skip(1) {
			eprintln!("  {} {}", "->".red().bold(), cause);
		}
		std::process::exit(1);
	}
}

fn cli_train(options: TrainOptions) -> Result<()> {
	// Start the progress view if enabled and train the model. However, we need to do some extra work to make panic messages display properly. The problem is that the progress view enables the terminal's alternative screen and returns to the default screen when it is dropped. However, if a panic occurs during training, it will be printed by the default panic hook while the alternative screen is active, and then the progress view will be dropped, causing the panic message to be immediately erased. To work around this, we create a custom panic hook that stores the panic message, wrap the progress view and training with `catch_unwind`, and then print the panic message if `catch_unwind` returns an `Err`. This ensure the progress view will be dropped before the panic message is displayed.
	pub static PANIC_INFO: Lazy<Mutex<Option<(String, Backtrace)>>> =
		Lazy::new(|| Mutex::new(None));
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_INFO.lock().unwrap().replace(value);
	}));
	let result = std::panic::catch_unwind(|| {
		let mut progress_view = if options.progress {
			ProgressView::new().ok()
		} else {
			None
		};
		tangram_core::train(
			tangram_util::id::Id::new(),
			options.file.as_deref(),
			options.file_train.as_deref(),
			options.file_test.as_deref(),
			&options.target,
			options.config.as_deref(),
			&mut |progress| {
				if let Some(progress_manager) = progress_view.as_mut() {
					progress_manager.update(progress)
				}
			},
		)
	});
	std::panic::set_hook(hook);
	let model = match result {
		Ok(result) => result,
		Err(_) => {
			let panic_info = PANIC_INFO.lock().unwrap();
			let (message, backtrace) = panic_info.as_ref().unwrap();
			Err(anyhow!("{}\n\n{:?}", message, backtrace))
		}
	}?;

	// Retrieve the output path from the command line arguments or generate a default.
	let output_path = match options.output.as_deref() {
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = options
				.file
				.as_ref()
				.unwrap()
				.file_stem()
				.unwrap()
				.to_str()
				.unwrap();
			Cow::Owned(available_path(&dir, csv_file_name, "tangram"))
		}
		Some(path) => Cow::Borrowed(path),
	};

	// Write the model to the output path.
	model
		.to_file(&output_path)
		.context("failed to write model to file")?;

	// Announce that everything worked!
	eprintln!("Your model was written to {}.", output_path.display());
	eprintln!(
		"For help making predictions in your code, read the docs at https://www.tangramhq.com/docs."
	);
	eprintln!(
		"To learn more about how your model works and set up production monitoring, run `tangram app`."
	);

	Ok(())
}

fn cli_app(options: AppOptions) -> Result<()> {
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		eprintln!("{}", panic_info.to_string());
	}));
	let mut runtime = tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap();
	runtime.block_on(tangram_app::run(tangram_app::Options {
		auth_enabled: options.auth_enabled,
		cookie_domain: options.cookie_domain,
		database_url: options.database_url.unwrap_or_else(default_database_url),
		database_max_connections: options.database_max_connections,
		host: options.host,
		model: options.model,
		port: options.port,
		sendgrid_api_token: options.sendgrid_api_token,
		stripe_publishable_key: options.stripe_publishable_key,
		stripe_secret_key: options.stripe_secret_key,
		url: options.url,
	}))?;
	std::panic::set_hook(hook);
	Ok(())
}

/// This function checks if a file with the given name and extension already exists at the path `base`, and if it does, it appends " 1", " 2", etc. to it until it finds a name that will not overwrite an existing file.
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

/// Retrieve the user data directory using the `dirs` crate.
fn data_dir() -> PathBuf {
	let tangram_data_dir = dirs::data_dir()
		.expect("failed to find user data directory")
		.join("tangram");
	std::fs::create_dir_all(&tangram_data_dir).unwrap_or_else(|_| {
		panic!(
			"failed to create tangram data directory in {}",
			tangram_data_dir.display()
		)
	});
	tangram_data_dir
}

/// Retrieve the default database url, which is a sqlite database in the user data directory.
fn default_database_url() -> Url {
	let tangram_database_path = data_dir().join("tangram.db");
	let url = format!(
		"sqlite:{}",
		tangram_database_path.to_str().unwrap().to_owned()
	);
	Url::parse(&url).unwrap()
}
