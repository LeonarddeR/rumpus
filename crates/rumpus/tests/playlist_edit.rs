//! Stepping through, removing from and clearing the playlist.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn next_track_delete_and_clear_edit_the_playlist() {
	let (base, fixtures) = common::prepare("playlist-edit");
	let mut app = common::launch(base, &[&fixtures.long_note, &fixtures.three_notes]);
	common::wait_ready(&app);
	let list = common::find_widget(app.pid, ControlType::List, common::PLAYLIST);
	common::wait_for_item_count(&list, 2);
	common::wait_for_selection(&list, &["long note"]);
	common::wait_for_status(app.pid, "Playing: long note");

	common::send_keys(app.pid, "{ctrl}{pgdn}");
	common::wait_for_selection(&list, &["three notes"]);
	common::wait_for_status(app.pid, "Playing: three notes");

	common::send_keys(app.pid, "{del}");
	common::wait_for_item_count(&list, 1);
	common::wait_for_status(app.pid, "Stopped");

	// Alt+F, C: File, Clear Playlist.
	common::send_keys(app.pid, "{alt}f");
	common::send_keys(app.pid, "c");
	common::wait_for_item_count(&list, 0);

	common::send_keys(app.pid, "{ctrl}q");
	let status = app.wait_exit(common::DEADLINE);
	assert!(status.success(), "app exited with {status:?}");
}
