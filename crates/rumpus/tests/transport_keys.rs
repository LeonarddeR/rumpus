//! Pause, seek, resume and stop from the keyboard while a song plays.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

/// The whole seconds of the elapsed part of `m:ss / m:ss`.
fn elapsed_seconds(position: &str) -> u64 {
	let (elapsed, _) = position.split_once(" / ").expect("elapsed / total");
	let (minutes, seconds) = elapsed.split_once(':').expect("m:ss");
	minutes.parse::<u64>().expect("minutes") * 60 + seconds.parse::<u64>().expect("seconds")
}

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn pause_seek_resume_and_stop_follow_the_keys() {
	let (base, fixtures) = common::prepare("transport-keys");
	let app = common::launch(base, &[&fixtures.long_note]);
	common::wait_ready(&app);
	common::wait_for_status(app.pid, "Playing: long note");
	let position = common::find_widget(app.pid, ControlType::Edit, common::POSITION);

	common::send_keys(app.pid, "{ctrl}p");
	common::wait_for_status(app.pid, "Paused: long note");
	common::wait_for_log(&app, "announce Paused");

	common::send_keys(app.pid, "{ctrl}{right}");
	let sought = common::wait_for_value_where(&position, |v| elapsed_seconds(v) >= 5);
	common::wait_for_log(&app, &format!("announce {sought}"));

	common::send_keys(app.pid, "{ctrl}p");
	common::wait_for_status(app.pid, "Playing: long note");

	common::send_keys(app.pid, "{ctrl}s");
	common::wait_for_status(app.pid, "Stopped");
	common::wait_for_log(&app, "announce Stopped");
	common::wait_for_value(&position, "0:00 / 0:10");
}
