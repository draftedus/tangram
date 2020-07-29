use anyhow::{Context, Result};
use backtrace::Backtrace;
use colored::*;
use once_cell::sync::Lazy;
use std::{
	borrow::Cow,
	panic::catch_unwind,
	path::{Path, PathBuf},
	sync::{
		mpsc::{channel, Receiver, Sender, TryRecvError},
		Mutex,
	},
	thread::{sleep, spawn, JoinHandle},
	time::Duration,
};
use tangram::id::Id;
use tangram::progress::Progress;
use term_ui::{Screen, Style};

mod app;
mod telemetry;

fn cli<'a, 'b>() -> clap::App<'a, 'b> {
	clap::App::new("tangram")
		.version(clap::crate_version!())
		.about("Train and deploy a machine learning model in minutes.")
		.setting(clap::AppSettings::SubcommandRequiredElseHelp)
		.subcommand(
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
		)
		.subcommand(
			clap::SubCommand::with_name("app")
				.about("run the reporting and monitoring app")
				.long_about("run the reporting and monitoring app"),
		)
}

fn main() {
	let matches = cli().get_matches();
	// std::panic::set_hook(Box::new(|panic_info| {
	// 	PANIC_INFO
	// 		.lock()
	// 		.unwrap()
	// 		.replace((panic_info.to_string(), Backtrace::new()));
	// }));
	// let track_run_handle = telemetry::track_run();
	// let result = catch_unwind(|| run(matches));
	// track_run_handle.join().unwrap();
	// let result = match result {
	// 	Err(_) => {
	// 		handle_panic();
	// 		Ok(())
	// 	}
	// 	Ok(r) => r,
	// };
	let result = run(matches);
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		error
			.chain()
			.skip(1)
			.for_each(|cause| eprintln!("  {} {}", "->".red().bold(), cause));
		std::process::exit(1);
	}
}

fn run(matches: clap::ArgMatches) -> Result<()> {
	match matches.subcommand() {
		("train", Some(train_matches)) => {
			let file_path = Path::new(train_matches.value_of("file").unwrap());
			let target_column_name = train_matches.value_of("target").unwrap();
			let config_path = train_matches.value_of("config").map(|s| Path::new(s));
			let output_path = train_matches.value_of("output").map(|s| Path::new(s));
			cli_train(file_path, target_column_name, config_path, output_path)
		}
		("app", Some(_)) => {
			let mut runtime = tokio::runtime::Runtime::new()?;
			runtime.block_on(app::start())
		}
		_ => unreachable!(),
	}
}

fn cli_train(
	file_path: &Path,
	target_column_name: &str,
	config_path: Option<&Path>,
	output_path: Option<&Path>,
) -> Result<()> {
	let progress = false;
	let mut progress_view = if progress {
		ProgressView::new().ok()
	} else {
		None
	};
	let mut update_progress = |p| {
		if let Some(progress_manager) = progress_view.as_mut() {
			progress_manager.update(p)
		}
	};

	let model = tangram::train(
		Id::new(),
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

pub static PANIC_INFO: Lazy<Mutex<Option<(String, Backtrace)>>> = Lazy::new(|| Mutex::new(None));

pub fn handle_panic() {
	let panic_info = PANIC_INFO.lock().unwrap();
	let (message, backtrace) = panic_info.as_ref().unwrap();
	let message = message.clone();
	let backtrace = format!("{:#?}", backtrace);
	drop(panic_info);
	if cfg!(debug_assertions) {
		eprintln!("{}", message);
		eprintln!("{}", backtrace);
	} else {
		let mut rl = rustyline::Editor::<()>::new();
		eprintln!("💩 The Tangram CLI crashed. Would you like to send the stack trace to Tangram?");
		let permission = rl.readline(r#"Enter "y" or "n": "#);
		let permission = match permission {
			Ok(s) => s,
			Err(_) => return,
		};
		let permission = permission.to_lowercase().starts_with('y');
		if !permission {
			return;
		}
		eprintln!("Enter your email address and an engineer will get back to you right away to fix things. If you want to be left alone, just hit enter.");
		let email = rl.readline("Enter email: ");
		let email = match email {
			Ok(s) => s,
			Err(_) => return,
		};
		telemetry::track_crash(message, backtrace, email.clone())
			.join()
			.ok();
		if email != "" {
			eprintln!("Thanks! We'll get back to you ASAP.");
		} else {
			eprintln!("Thanks! We'll fix this bug ASAP.");
		}
	}
}

pub struct ProgressView {
	thread: Option<JoinHandle<()>>,
	sender: Option<Sender<Option<Progress>>>,
}

impl ProgressView {
	pub fn new() -> Result<Self> {
		let (sender, receiver) = channel::<Option<Progress>>();
		let mut screen = Screen::open()?;
		screen.hide_cursor().unwrap();
		screen.flush().unwrap();
		let thread = Some(spawn(move || thread_main(screen, receiver)));
		Ok(Self {
			thread,
			sender: Some(sender),
		})
	}
	pub fn update(&mut self, progress: Progress) {
		self.sender.as_ref().unwrap().send(Some(progress)).unwrap();
	}
}

impl Drop for ProgressView {
	fn drop(&mut self) {
		self.sender.take().unwrap().send(None).unwrap();
		self.thread.take().unwrap().join().unwrap();
	}
}

fn thread_main(mut screen: Screen, receiver: Receiver<Option<Progress>>) {
	let mut progress = None;
	loop {
		match receiver.try_recv() {
			Err(TryRecvError::Empty) => {}
			Err(TryRecvError::Disconnected) => unreachable!(),
			Ok(None) => break,
			Ok(Some(new_progress)) => progress = Some(new_progress),
		};
		if let Some(progress) = progress.as_ref() {
			screen.clear().unwrap();
			screen.put_str(0, 0, Style::default(), &format!("{:?}", progress));
			screen.flush().unwrap();
			panic!("epic sadness");
		}
		sleep(Duration::from_millis(15));
	}
}

// use num_traits::ToPrimitive;
// use std::io::{Result, Write};
// use std::time::{Duration, Instant};

// pub struct Options {
// 	pub formatter: Formatter,
// 	pub term_width: usize,
// 	pub start: Instant,
// 	pub total: u64,
// }

// #[derive(Copy, Clone)]
// pub enum Formatter {
// 	Normal,
// 	Bytes,
// 	CustomUnit(&'static str),
// }

// #[cfg(unix)]
// mod chars {
// 	pub const LEFT_CHAR: &str = "|";
// 	pub const RIGHT_CHAR: &str = "|";
// 	pub const BAR_CHARS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
// }

// #[cfg(windows)]
// mod chars {
// 	pub const LEFT_CHAR: &str = "[";
// 	pub const RIGHT_CHAR: &str = "]";
// 	pub const BAR_CHARS: [char; 9] = [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'];
// }

// pub fn draw<W: Write>(f: &mut W, current: u64, options: &Options) -> Result<()> {
// 	draw_bar(f, current, options)?;
// 	draw_text(f, current, options)?;
// 	Ok(())
// }

// fn draw_bar<W: Write>(f: &mut W, current: u64, options: &Options) -> Result<()> {
// 	let bar_width = usize::min(options.term_width, 80) - 2;
// 	let fraction = (current.to_f64().unwrap()) / (options.total.to_f64().unwrap());
// 	write!(f, "{}", chars::LEFT_CHAR).unwrap();
// 	for i in 0..bar_width {
// 		let fill_fraction = f64::max(
// 			0.0,
// 			f64::min(
// 				fraction * bar_width.to_f64().unwrap() - i.to_f64().unwrap(),
// 				1.0,
// 			),
// 		);
// 		let char_index = (fill_fraction * (chars::BAR_CHARS.len() - 1).to_f64().unwrap())
// 			.floor()
// 			.to_usize()
// 			.unwrap();
// 		write!(f, "{}", chars::BAR_CHARS[char_index])?;
// 	}
// 	writeln!(f, "{}", chars::RIGHT_CHAR)?;
// 	Ok(())
// }

// fn draw_text<W: Write>(f: &mut W, current: u64, options: &Options) -> Result<()> {
// 	let fraction = (current.to_f64().unwrap()) / (options.total.to_f64().unwrap());
// 	let elapsed = options.start.elapsed();
// 	let elapsed_secs = elapsed.as_secs().to_f64().unwrap()
// 		+ elapsed.subsec_nanos().to_f64().unwrap() / 1_000_000_000f64;
// 	let eta = if fraction > std::f64::EPSILON {
// 		Some(Duration::from_secs(
// 			((elapsed_secs / fraction) - elapsed_secs)
// 				.floor()
// 				.to_u64()
// 				.unwrap(),
// 		))
// 	} else {
// 		None
// 	};
// 	match &options.formatter {
// 		Formatter::Normal => {
// 			write!(f, "{} / {}", current, options.total)?;
// 		}
// 		Formatter::CustomUnit(s) => {
// 			write!(f, "{} / {}{}", current, options.total, s)?;
// 		}
// 		Formatter::Bytes => {
// 			write!(
// 				f,
// 				"{} / {}",
// 				DisplayBytes(current),
// 				DisplayBytes(options.total)
// 			)?;
// 		}
// 	};
// 	write!(f, " {} elapsed", DisplayDuration(elapsed))?;
// 	if let Some(eta) = eta {
// 		write!(f, " {} remaining", DisplayDuration(eta))?;
// 	}
// 	writeln!(f)?;
// 	Ok(())
// }

// pub struct DisplayBytes(pub u64);

// impl std::fmt::Display for DisplayBytes {
// 	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
// 		let value = self.0;
// 		if value >= 1_000_000_000_000_000 {
// 			write!(f, "{}PB", value / 1_000_000_000_000_000)
// 		} else if value >= 1_000_000_000_000 {
// 			write!(f, "{}TB", value / 1_000_000_000_000)
// 		} else if value >= 1_000_000_000 {
// 			write!(f, "{}GB", value / 1_000_000_000)
// 		} else if value >= 1_000_000 {
// 			write!(f, "{}MB", value / 1_000_000)
// 		} else if value >= 1_000 {
// 			write!(f, "{}KB", value / 1_000)
// 		} else {
// 			write!(f, "{}B", value)
// 		}
// 	}
// }

// pub struct DisplayDuration(pub Duration);

// impl std::fmt::Display for DisplayDuration {
// 	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
// 		let milliseconds = self.0.as_millis();
// 		let seconds = self.0.as_secs();
// 		let minutes = seconds / 60;
// 		let hours = seconds / (60 * 60);
// 		let days = seconds / (24 * 60 * 60);
// 		if days >= 1 {
// 			write!(
// 				f,
// 				"{}d {}h",
// 				days,
// 				(seconds - days * (24 * 60 * 60)) / (60 * 60)
// 			)
// 		} else if hours >= 1 {
// 			write!(f, "{}h {}m", hours, (seconds - hours * (60 * 60)) / 60)
// 		} else if minutes >= 1 {
// 			write!(f, "{}m {}s", minutes, (seconds - minutes * 60))
// 		} else if seconds >= 1 {
// 			write!(f, "{}s", seconds)
// 		} else if milliseconds >= 1 {
// 			write!(f, "0s {}ms", milliseconds)
// 		} else {
// 			write!(f, "0ms")
// 		}
// 	}
// }
