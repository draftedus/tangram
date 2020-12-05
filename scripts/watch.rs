use notify::Watcher;
use std::{path::PathBuf, sync::mpsc::channel, time::Duration};
use tangram_util::error::Result;

pub fn watch(
	watch_paths: Vec<PathBuf>,
	ignore_paths: Vec<PathBuf>,
	cmd: String,
	args: Vec<String>,
) -> Result<()> {
	let mut process = ChildProcess::new(cmd, args);
	process.start()?;
	let (tx, rx) = channel();
	let mut watcher = notify::watcher(tx, Duration::from_secs_f32(0.1)).unwrap();
	for path in watch_paths.iter() {
		watcher.watch(path, notify::RecursiveMode::Recursive)?;
	}
	loop {
		let event = rx.recv()?;
		let paths = match event {
			notify::DebouncedEvent::NoticeWrite(path) => vec![path],
			notify::DebouncedEvent::NoticeRemove(path) => vec![path],
			notify::DebouncedEvent::Create(path) => vec![path],
			notify::DebouncedEvent::Write(path) => vec![path],
			notify::DebouncedEvent::Chmod(path) => vec![path],
			notify::DebouncedEvent::Remove(path) => vec![path],
			notify::DebouncedEvent::Rename(path_a, path_b) => vec![path_a, path_b],
			notify::DebouncedEvent::Rescan => Vec::new(),
			notify::DebouncedEvent::Error(_, path) => {
				path.map(|path| vec![path]).unwrap_or_else(Vec::new)
			}
		};
		let should_restart = paths.iter().any(|path| {
			!ignore_paths
				.iter()
				.any(|ignore_path| path.starts_with(ignore_path))
		});
		if should_restart {
			process.restart()?;
		}
	}
}

struct ChildProcess {
	cmd: String,
	args: Vec<String>,
	process: Option<std::process::Child>,
}

impl ChildProcess {
	pub fn new(cmd: String, args: Vec<String>) -> ChildProcess {
		ChildProcess {
			cmd,
			args,
			process: None,
		}
	}

	pub fn start(&mut self) -> Result<()> {
		let process = std::process::Command::new(&self.cmd)
			.args(&self.args)
			.spawn()?;
		self.process.replace(process);
		Ok(())
	}

	pub fn stop(&mut self) -> Result<()> {
		if let Some(mut process) = self.process.take() {
			process.kill()?;
			process.wait()?;
		}
		Ok(())
	}

	pub fn restart(&mut self) -> Result<()> {
		self.stop()?;
		self.start()?;
		Ok(())
	}
}

impl Drop for ChildProcess {
	fn drop(&mut self) {
		self.stop().unwrap();
	}
}

// let mut cmd = vec![cmd];
// cmd.extend(args);
// watchexec::run(
// 	watchexec::ArgsBuilder::default()
// 		.cmd(cmd)
// 		.paths(watch_paths)
// 		.ignores(
// 			ignore_paths
// 				.into_iter()
// 				.map(|path| path.to_str().unwrap().to_owned())
// 				.collect::<Vec<String>>(),
// 		)
// 		.restart(true)
// 		.build()
// 		.unwrap(),
// )?;
// Ok(())
