use self::progress_view::ProgressView;
use crate::TrainArgs;
use backtrace::Backtrace;
use once_cell::sync::Lazy;
use std::{
	path::{Path, PathBuf},
	sync::Mutex,
};
use tangram_util::{err, error::Result};

mod progress_view;

#[cfg(feature = "train")]
pub fn train(args: TrainArgs) -> Result<()> {
	// Start the progress view if enabled and train the model. However, we need to do some extra work to make panic messages display properly. The problem is that the progress view enables the terminal's alternative screen and returns to the default screen when it is dropped. However, if a panic occurs during training, it will be printed by the default panic hook while the alternative screen is active, and then the progress view will be dropped, causing the panic message to be immediately erased. To work around this, we create a custom panic hook that stores the panic message, wrap the progress view and training with `catch_unwind`, and then print the panic message if `catch_unwind` returns an `Err`. This ensures that the progress view will be dropped before the panic message is displayed.
	static PANIC_MESSAGE_AND_BACKTRACE: Lazy<Mutex<Option<(String, Backtrace)>>> =
		Lazy::new(|| Mutex::new(None));
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.lock().unwrap().replace(value);
	}));
	let result = std::panic::catch_unwind(|| {
		let mut progress_view = if args.progress {
			ProgressView::new().ok()
		} else {
			None
		};
		tangram_core::train(
			tangram_util::id::Id::new(),
			args.file.as_deref(),
			args.file_train.as_deref(),
			args.file_test.as_deref(),
			&args.target,
			args.config.as_deref(),
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
			let panic_info = PANIC_MESSAGE_AND_BACKTRACE.lock().unwrap();
			let (message, backtrace) = panic_info.as_ref().unwrap();
			Err(err!("{}\n{:?}", message, backtrace))
		}
	}?;

	// Retrieve the output path from the command line arguments or generate a default.
	let output_path = match args.output {
		Some(output) => output,
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = args
				.file
				.as_ref()
				.unwrap()
				.file_stem()
				.unwrap()
				.to_str()
				.unwrap();
			available_path(&dir, csv_file_name, "tangram")?
		}
	};

	// Write the model to the output path.
	model.to_file(&output_path)?;

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

/// This function checks if a file with the given name and extension already exists at the path `base`, and if it does, it appends " 1", " 2", etc. to it until it finds a name that will not overwrite an existing file.
fn available_path(dir: &Path, name: &str, extension: &str) -> Result<PathBuf> {
	let mut i = 0;
	loop {
		let mut path = PathBuf::from(dir);
		let mut filename = String::new();
		filename.push_str(name);
		if i > 0 {
			filename.push(' ');
			filename.push_str(&i.to_string());
		}
		filename.push('.');
		filename.push_str(extension);
		path.push(filename);
		match std::fs::metadata(&path) {
			// If a file at the path does not exist, return the path.
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(path);
			}
			Err(error) => return Err(error.into()),
			// If a file at the path exists, try the next number.
			Ok(_) => {
				i += 1;
				continue;
			}
		}
	}
}
