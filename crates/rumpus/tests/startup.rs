//! Startup against the real GUI: focus, readiness once the service is probed, and a clean exit.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn playlist_is_focused_and_ctrl_q_exits_cleanly() {
	let mut app = common::launch_empty("startup");
	common::wait_ready(&app);

	common::wait_for_focus(app.pid, common::DEADLINE, common::is_playlist);
	common::wait_for_status(app.pid, "Stopped");
	assert!(app.config().contains("loopback_a"), "config: {}", app.config());

	common::send_keys(app.pid, "{ctrl}q");
	let status = app.wait_exit(common::DEADLINE);
	assert!(status.success(), "app exited with {status:?}");
	assert!(app.logs().contains("starting"), "logs:\n{}", app.logs());
}
