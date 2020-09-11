/*!
This file contains the main entrypoint to the tangram cli.
*/

use anyhow::{format_err, Context, Result};
use clap::Clap;
use colored::*;
use once_cell::sync::Lazy;
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
	sync::Mutex,
};

#[cfg(feature = "train")]
use progress_view::ProgressView;

#[cfg(feature = "app")]
use url::Url;

#[cfg(feature = "train")]
mod progress_view;

#[cfg(feature = "app")]
mod app;

#[derive(Clap)]
#[clap(
	about = "Train and deploy a machine learning model in minutes.",
	setting = clap::AppSettings::DisableHelpSubcommand,
)]
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
#[clap(long_about = "train a model from a csv file")]
struct TrainOptions {
	#[clap(short, long, about = "the path to your .csv file")]
	file: PathBuf,
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
	// Start the progress view if enabled and train the model. However, we need to do some extra work to make panic messages display properly. The problem is that the progress view enables the terminal's alternative screen and returns to the default screen when it is dropped. However, if a panic occurs during training, it will be printed by the default panic hook while the alternative screen is active, and then the progress view will be dropped, causing the panic message to be immediately erased. To work around this, we create a custom panic hook that stores the panic message, wrap the progress view and training with `catch_unwind`, and then print the panic message if `catch_unwind` returns an `Err`. This ensure the progress view will be dropped before the panic message is displayed.
	pub static PANIC_INFO: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		PANIC_INFO.lock().unwrap().replace(panic_info.to_string());
	}));
	let result = std::panic::catch_unwind(|| {
		let mut progress_view = if options.progress {
			ProgressView::new().ok()
		} else {
			None
		};
		tangram::train(
			tangram::id::Id::new(),
			&options.file,
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
		Err(_) => Err(format_err!(
			"{}",
			PANIC_INFO.lock().unwrap().as_ref().unwrap()
		)),
	}?;

	// Retrieve the output path from the command line arguments or generate a default.
	let output_path = match options.output.as_deref() {
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = options.file.file_stem().unwrap().to_str().unwrap();
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

#[cfg(feature = "app")]
fn cli_app(options: AppOptions) -> Result<()> {
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		eprintln!("{}", panic_info.to_string());
	}));
	let mut runtime = tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()?;
	runtime.block_on(app::run(app::AppOptions {
		auth_enabled: options.auth_enabled,
		cookie_domain: options.cookie_domain,
		database_url: options.database_url,
		host: options.host,
		port: options.port,
		sendgrid_api_token: options.sendgrid_api_token,
		stripe_publishable_key: options.stripe_publishable_key,
		stripe_secret_key: options.stripe_secret_key,
		url: options.url,
	}))?;
	std::panic::set_hook(hook);
	Ok(())
}

/**
This function checks if a file with the given name and extension already exists at the path `base`, and if it does, it appends " 1", " 2", etc. to it until it finds a name that will not overwrite an existing file.
*/
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
