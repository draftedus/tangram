use self::generated::*;
use crate::env_var_enabled;
use buffy::prelude::*;
use itertools::Itertools;

mod generated;

fn track(event: Event) -> std::thread::JoinHandle<()> {
	std::thread::spawn(move || {
		if env_var_enabled("TANGRAM_TELEMETRY_OPTOUT") || cfg!(debug_assertions) {
			return;
		}
		let base_url = std::env::var("TANGRAM_TELEMETRY_BASE_URL")
			.unwrap_or_else(|_| "https://telemetry.tangramhq.com".to_owned());
		let client = reqwest::blocking::Client::new();
		let url = base_url + "/track/cli";
		let data = buffy::to_vec(&event).unwrap();
		client.post(&url).body(data).send().ok();
	})
}

fn operating_system() -> OperatingSystem {
	if cfg!(target_os = "linux") {
		OperatingSystem::Linux
	} else if cfg!(target_os = "macos") {
		OperatingSystem::MacOs
	} else if cfg!(target_os = "windows") {
		OperatingSystem::Windows
	} else {
		OperatingSystem::Unknown
	}
}

pub fn track_run() -> std::thread::JoinHandle<()> {
	let event = Event::Run(RunEvent {
		host_info: Present(HostInfo {
			operating_system: Present(operating_system()),
			cli_version: Present(clap::crate_version!().to_owned()),
			..Default::default()
		}),
		command: Present(std::env::args().join(" ")),
		..Default::default()
	});
	track(event)
}

pub fn track_crash(
	message: String,
	backtrace: String,
	email: String,
) -> std::thread::JoinHandle<()> {
	let event = Event::Crash(CrashEvent {
		host_info: Present(HostInfo {
			operating_system: Present(operating_system()),
			cli_version: Present(clap::crate_version!().to_owned()),
			..Default::default()
		}),
		command: Present(std::env::args().join(" ")),
		backtrace: Present(backtrace),
		message: Present(message),
		email: Present(email),
		..Default::default()
	});
	track(event)
}
