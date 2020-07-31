use anyhow::{Context, Result};
use colored::*;
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
};

#[cfg(feature = "train")]
mod progress;

fn cli<'a, 'b>() -> clap::App<'a, 'b> {
	let mut app = clap::App::new("tangram")
		.version(clap::crate_version!())
		.about("Train and deploy a machine learning model in minutes.")
		.setting(clap::AppSettings::SubcommandRequiredElseHelp);
	if cfg!(feature = "train") {
		app = app.subcommand(
			clap::SubCommand::with_name("train")
				.about("train a model")
				.long_about("train a model from a csv file with a specified target column")
				.arg(
					clap::Arg::with_name("file")
						.help("the path to your .csv file")
						.long("file")
						.short("f")
						.takes_value(true)
						.required(true),
				)
				.arg(
					clap::Arg::with_name("target")
						.help("the name of the column to predict")
						.long("target")
						.short("t")
						.takes_value(true)
						.required(true),
				)
				.arg(
					clap::Arg::with_name("config")
						.help("the path to a config file")
						.long("config")
						.short("c")
						.takes_value(true),
				)
				.arg(
					clap::Arg::with_name("output")
						.help("the path to write the output to")
						.long("output")
						.short("o")
						.takes_value(true),
				),
		);
	}
	if cfg!(feature = "app") {
		app = app.subcommand(
			clap::SubCommand::with_name("app")
				.about("run the reporting and monitoring app")
				.long_about("run the reporting and monitoring app"),
		)
	}
	app
}

fn main() {
	let matches = cli().get_matches();
	let result = match matches.subcommand() {
		#[cfg(feature = "train")]
		("train", Some(train_matches)) => {
			let file_path = Path::new(train_matches.value_of("file").unwrap());
			let target_column_name = train_matches.value_of("target").unwrap();
			let config_path = train_matches.value_of("config").map(|s| Path::new(s));
			let output_path = train_matches.value_of("output").map(|s| Path::new(s));
			cli_train(file_path, target_column_name, config_path, output_path)
		}
		#[cfg(feature = "app")]
		("app", Some(_)) => {
			let mut runtime = tokio::runtime::Runtime::new().unwrap();
			runtime.block_on(tangram_app::start())
		}
		_ => unreachable!(),
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
fn cli_train(
	file_path: &Path,
	target_column_name: &str,
	config_path: Option<&Path>,
	output_path: Option<&Path>,
) -> Result<()> {
	let progress = false;
	let mut progress_view = if progress {
		progress::ProgressView::new().ok()
	} else {
		None
	};
	let mut update_progress = |p| {
		if let Some(progress_manager) = progress_view.as_mut() {
			progress_manager.update(p)
		}
	};

	let model = tangram::train(
		tangram::id::Id::new(),
		file_path,
		target_column_name,
		config_path,
		&mut update_progress,
	)?;

	drop(progress_view);

	let output_path = match output_path {
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = file_path.file_stem().unwrap().to_str().unwrap();
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

pub fn available_path(base: &Path, name: &str, extension: &str) -> PathBuf {
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

pub fn env_var_enabled(var: &str) -> bool {
	match std::env::var(var) {
		Err(_) => false,
		Ok(v) => v == "1" || v == "true",
	}
}
