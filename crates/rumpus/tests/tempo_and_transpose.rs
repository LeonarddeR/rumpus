//! Tempo and transpose shortcuts update the spin controls and announce the change.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn shortcuts_move_the_spin_controls_and_announce() {
	let app = common::launch_empty("tempo-transpose");
	common::wait_ready(&app);
	let tempo = common::find_widget(app.pid, ControlType::Edit, common::TEMPO_EDIT);
	let transpose = common::find_widget(app.pid, ControlType::Edit, common::TRANSPOSE_EDIT);

	common::send_keys(app.pid, "{ctrl}{up}");
	common::wait_for_value(&tempo, "105");
	common::wait_for_log(&app, "announce Tempo 105%");

	common::send_keys(app.pid, "{ctrl}{backspace}");
	common::wait_for_value(&tempo, "100");
	common::wait_for_log(&app, "announce Tempo 100%");

	common::send_keys(app.pid, "{ctrl}{shift}{up}");
	common::wait_for_value(&transpose, "1");
	common::wait_for_log(&app, "announce Transpose up 1 semitone");

	common::send_keys(app.pid, "{ctrl}{shift}{backspace}");
	common::wait_for_value(&transpose, "0");
	common::wait_for_log(&app, "announce Transpose off");
}
