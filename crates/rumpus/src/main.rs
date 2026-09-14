#![cfg_attr(not(test), windows_subsystem = "windows")]

mod announce;
mod app;
mod dialogs;
mod menu;
mod midi;
mod player;

use std::path::PathBuf;

use rumpus_core::config::AppConfig;

const LOG_FILE: &str = "rumpus.log";

fn main() {
	let (config, config_warning) = AppConfig::load();
	init_logging();
	tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
	let files: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
	if let Err(e) = wxdragon::main(move |_| {
		let app = app::App::new(config, config_warning, files);
		let app: &'static app::App = Box::leak(Box::new(app));
		app::store_app(app);
		app.start();
	}) {
		tracing::error!("wxdragon: {e}");
		eprintln!("rumpus: {e}");
		std::process::exit(1);
	}
}

/// Logs to a file in the config directory, since a windows-subsystem process has no console.
fn init_logging() {
	let Some(dir) = AppConfig::dir() else { return };
	if std::fs::create_dir_all(&dir).is_err() {
		return;
	}
	let Ok(file) = std::fs::File::create(dir.join(LOG_FILE)) else { return };
	let _ = tracing_subscriber::fmt().with_writer(file).with_ansi(false).try_init();
}
