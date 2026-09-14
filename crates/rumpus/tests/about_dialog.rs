//! The About dialog opens from the Help menu and Escape returns to the playlist.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn help_menu_opens_about_and_escape_closes_it() {
	let mut app = common::launch_empty("about-dialog");
	common::wait_ready(&app);

	common::send_keys(app.pid, "{alt}h");
	common::send_keys(app.pid, "a");
	common::find_window(app.pid, "About Rumpus");

	common::send_keys(app.pid, "{esc}");
	common::wait_for_focus(app.pid, common::DEADLINE, common::is_playlist);
	assert!(app.is_running(), "app exited after Escape");
}
