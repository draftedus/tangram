use anyhow::Result;
use std::{
	sync::mpsc::{channel, Receiver, Sender, TryRecvError},
	thread::{sleep, spawn, JoinHandle},
	time::Duration,
};
use tangram::progress::Progress;
use term_ui::{Screen, Style};

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

// let mut update_progress = |progress| match progress_view.as_mut() {
// 	Some(progress_manager) => progress_manager.update(progress),
// 	None => match progress {
// 		tangram::progress::Progress::Loading(_) => println!("Loading Data"),
// 		tangram::progress::Progress::Shuffling => {
// 			print!("\x1b[1A\x1b[0K");
// 			println!("Loading Data \x1b[1;92m✓\x1b[0m");
// 			println!("Shuffling Data");
// 		}
// 		tangram::progress::Progress::Stats(p) => match p {
// 			tangram::progress::StatsProgress::DatasetStats(_) => {
// 				print!("\x1b[1A\x1b[0K");
// 				println!("Shuffling Data \x1b[1;92m✓\x1b[0m");
// 				println!("Computing Stats step 1 of 2");
// 			}
// 			tangram::progress::StatsProgress::HistogramStats(_) => {
// 				print!("\x1b[1A\x1b[0K");
// 				println!("Computing Stats step 2 of 2")
// 			}
// 		},
// 		tangram::progress::Progress::Training(p) => match p {
// 			tangram::progress::GridTrainProgress {
// 				current,
// 				total,
// 				grid_item_progress,
// 			} => match grid_item_progress {
// 				tangram::progress::TrainProgress::ComputingFeatures(p) => {
// 					print!("\x1b[1A\x1b[0K");
// 					if current == 1 {
// 						println!("Computing Stats \x1b[1;92m✓\x1b[0m");
// 					}
// 					println!("Training model {} of {} {:?}", current, total, p);
// 				}
// 				tangram::progress::TrainProgress::TrainingModel(p) => {
// 					print!("\x1b[1A\x1b[0K");
// 					println!("Training model {} of {} {:?}", current, total, p);
// 				}
// 				tangram::progress::TrainProgress::ComputingModelComparisonMetrics(p) => {
// 					print!("\x1b[1A\x1b[0K");
// 					println!("Training model {} of {} {:?}", current, total, p);
// 				}
// 			},
// 		},
// 		tangram::progress::Progress::Testing => {
// 			print!("\x1b[1A\x1b[0K");
// 			println!("Training \x1b[1;92m✓\x1b[0m",);
// 			println!("Testing best model");
// 		}
// 	},
// };

// println!("\x1b[1A\x1b[0KTesting best model \x1b[1;92m✓\x1b[0m",);
