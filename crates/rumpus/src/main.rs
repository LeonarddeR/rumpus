#![cfg_attr(not(test), windows_subsystem = "windows")]

mod announce;
mod app;
mod dialogs;
mod ipc;
mod menu;
mod midi;
mod player;
#[allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	clippy::all,
	clippy::pedantic,
	clippy::nursery
)]
mod win32;

use std::{ffi::OsString, path::PathBuf};

use rumpus_core::config::AppConfig;
use tracing_subscriber::EnvFilter;

use crate::midi::backend;

const LOG_FILE: &str = "rumpus.log";
const PROBE_FLAG: &str = "--probe";

fn main() {
	let args: Vec<OsString> = std::env::args_os().skip(1).collect();
	if args.len() == 1 && args[0] == PROBE_FLAG {
		std::process::exit(probe());
	}
	let files: Vec<PathBuf> = args.into_iter().map(|arg| absolute(PathBuf::from(arg))).collect();
	let (server, ipc_warning) = match ipc::claim() {
		Ok(server) => (Some(server), None),
		Err(ipc::ClaimError::AnotherInstance) => match ipc::forward(&files) {
			Ok(()) => return,
			Err(e) => (None, Some(format!("forwarding to the running instance failed: {e}"))),
		},
		Err(ipc::ClaimError::Os(e)) => (None, Some(format!("creating the single-instance pipe failed: {e}"))),
	};
	let (config, config_warning) = AppConfig::load();
	init_logging();
	tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
	if let Some(warning) = ipc_warning {
		tracing::warn!("ipc: {warning}");
	}
	if let Err(e) = wxdragon::main(move |_| {
		let app = app::App::new(config, config_warning, files);
		let app: &'static app::App = Box::leak(Box::new(app));
		app::store_app(app);
		app.start(server);
	}) {
		tracing::error!("wxdragon: {e}");
		eprintln!("rumpus: {e}");
		std::process::exit(1);
	}
}

/// Runs the Windows MIDI Services check on its own and reports it as an exit code.
fn probe() -> i32 {
	let result =
		backend::init().map_err(|e| backend::Unavailable::NotInstalled(e.message())).and_then(|()| backend::probe());
	backend::probe_exit_code(&result)
}

fn absolute(path: PathBuf) -> PathBuf {
	std::path::absolute(&path).unwrap_or(path)
}

/// Logs to a file in the config directory, since a windows-subsystem process has no console;
/// `RUST_LOG` selects the level.
fn init_logging() {
	let Some(dir) = AppConfig::dir() else { return };
	if std::fs::create_dir_all(&dir).is_err() {
		return;
	}
	let Ok(file) = std::fs::File::create(dir.join(LOG_FILE)) else { return };
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	let _ = tracing_subscriber::fmt().with_env_filter(filter).with_writer(file).with_ansi(false).try_init();
}
