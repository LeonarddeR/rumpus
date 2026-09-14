//! A file on the command line is listed, played and reported until the playlist ends.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn file_from_the_command_line_plays_to_the_end() {
	let (base, fixtures) = common::prepare("plays-file");
	let mut app = common::launch(base, &[&fixtures.three_notes]);
	common::wait_ready(&app);

	let list = common::find_widget(app.pid, ControlType::List, common::PLAYLIST);
	common::wait_for_selection(&list, &["three notes"]);
	common::wait_for_log(&app, "announce Playing three notes");

	let position = common::find_widget(app.pid, ControlType::Edit, common::POSITION);
	common::wait_for_value_where(&position, |v| v.ends_with("/ 0:01"));

	common::wait_for_log(&app, "End of playlist");
	common::wait_for_status(app.pid, "Stopped");
	assert!(app.is_running(), "app exited after the playlist ended");
}
