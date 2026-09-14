//! Picking another output from the Device menu switches and persists it.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive desktop and Windows MIDI
//! Services. Run explicitly: `cargo test -p rumpus -- --ignored`
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop and Windows MIDI Services"]
fn picking_the_second_output_saves_it() {
	let app = common::launch_empty("device-menu");
	common::wait_ready(&app);
	common::wait_for_config_where(&app, |c| c.contains("loopback_a"));

	common::send_keys(app.pid, "{alt}d");
	let menu = common::wait_for_popup(app.pid, "Device");
	common::find_widget_in(&menu, ControlType::MenuItem, common::LOOPBACK_A);
	common::find_widget_in(&menu, ControlType::MenuItem, common::LOOPBACK_B);

	// The menu opens on the checked Loopback A; Down moves to Loopback B.
	common::press(app.pid, &[], common::VK_DOWN);
	common::press(app.pid, &[], common::VK_RETURN);

	common::wait_for_config_where(&app, |c| c.contains("loopback_b"));
	common::wait_for_focus(app.pid, common::DEADLINE, common::is_playlist);
	let tempo = common::find_widget(app.pid, ControlType::Spinner, common::TEMPO_SPINNER);
	common::wait_for_enabled(&tempo, true);
}
