//! A second instance hands its files to the running one and exits.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn second_instance_forwards_its_file_and_exits() {
	let (base, fixtures) = common::prepare("single-instance");
	let mut app = common::launch(base, &[]);
	common::wait_ready(&app);

	let second = app.another_instance().arg(&fixtures.three_notes).output().expect("run the second instance");
	assert!(second.status.success(), "second instance exited with {:?}", second.status);

	common::wait_for_log(&app, "ipc");
	let list = common::find_widget(app.pid, ControlType::List, common::PLAYLIST);
	common::wait_for_selection(&list, &["three notes"]);
	common::wait_for_log(&app, "announce Playing three notes");
	assert!(app.is_running(), "app exited after the forwarded open");
}
