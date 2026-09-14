//! The menu bar; every command lives here so its shortcut is discoverable.

use wxdragon::prelude::*;

pub mod ids {
	pub const OPEN_FILES: i32 = 5001;
	pub const ADD_FOLDER: i32 = 5002;
	pub const REMOVE: i32 = 5003;
	pub const CLEAR: i32 = 5004;
	pub const EXIT: i32 = 5005;
	pub const PLAY_PAUSE: i32 = 5010;
	pub const STOP: i32 = 5011;
	pub const PREVIOUS: i32 = 5012;
	pub const NEXT: i32 = 5013;
	pub const SEEK_BACK: i32 = 5014;
	pub const SEEK_FORWARD: i32 = 5015;
	pub const SEEK_BACK_FAR: i32 = 5016;
	pub const SEEK_FORWARD_FAR: i32 = 5017;
	pub const ANNOUNCE_POSITION: i32 = 5018;
	pub const TEMPO_DOWN: i32 = 5020;
	pub const TEMPO_UP: i32 = 5021;
	pub const TEMPO_RESET: i32 = 5022;
	pub const TRANSPOSE_DOWN: i32 = 5030;
	pub const TRANSPOSE_UP: i32 = 5031;
	pub const TRANSPOSE_RESET: i32 = 5032;
	pub const REFRESH_DEVICES: i32 = 5040;
	pub const ABOUT: i32 = 5050;

	/// Commands that need a loaded output and are disabled until the service is ready.
	pub const PLAYBACK: [i32; 15] = [
		PLAY_PAUSE,
		STOP,
		PREVIOUS,
		NEXT,
		SEEK_BACK,
		SEEK_FORWARD,
		SEEK_BACK_FAR,
		SEEK_FORWARD_FAR,
		ANNOUNCE_POSITION,
		TEMPO_DOWN,
		TEMPO_UP,
		TEMPO_RESET,
		TRANSPOSE_DOWN,
		TRANSPOSE_UP,
		TRANSPOSE_RESET,
	];
}

/// Menu items as (id, label with `\t` shortcut, status bar help).
const FILE_ITEMS: &[(i32, &str, &str)] = &[
	(ids::OPEN_FILES, "&Open Files...\tCtrl+O", "Add MIDI files to the playlist"),
	(ids::ADD_FOLDER, "Add &Folder...\tCtrl+Shift+O", "Add every MIDI file in a folder to the playlist"),
	(ids::REMOVE, "&Remove From Playlist\tDel", "Remove the selected file from the playlist"),
	(ids::CLEAR, "&Clear Playlist", "Remove every file from the playlist"),
];

const PLAYBACK_ITEMS: &[(i32, &str, &str)] = &[
	(ids::PLAY_PAUSE, "&Play/Pause\tCtrl+P", "Start, pause or resume playback"),
	(ids::STOP, "&Stop\tCtrl+S", "Stop playback and return to the start"),
	(ids::PREVIOUS, "Pre&vious Track\tCtrl+PgUp", "Play the previous file in the playlist"),
	(ids::NEXT, "&Next Track\tCtrl+PgDn", "Play the next file in the playlist"),
	(ids::SEEK_BACK, "Seek &Back 5 Seconds\tCtrl+Left", "Move the position back five seconds"),
	(ids::SEEK_FORWARD, "Seek &Forward 5 Seconds\tCtrl+Right", "Move the position forward five seconds"),
	(ids::SEEK_BACK_FAR, "Seek Back 30 Seconds\tCtrl+Shift+Left", "Move the position back thirty seconds"),
	(ids::SEEK_FORWARD_FAR, "Seek Forward 30 Seconds\tCtrl+Shift+Right", "Move the position forward thirty seconds"),
	(ids::ANNOUNCE_POSITION, "Announce Pos&ition\tCtrl+I", "Speak the elapsed and total time"),
];

const TEMPO_ITEMS: &[(i32, &str, &str)] = &[
	(ids::TEMPO_DOWN, "&Slower\tCtrl+Down", "Lower the tempo by five percent"),
	(ids::TEMPO_UP, "&Faster\tCtrl+Up", "Raise the tempo by five percent"),
	(ids::TEMPO_RESET, "&Reset Tempo\tCtrl+0", "Return to the original tempo"),
];

const TRANSPOSE_ITEMS: &[(i32, &str, &str)] = &[
	(ids::TRANSPOSE_DOWN, "&Down a Semitone\tCtrl+Shift+Down", "Transpose down by one semitone"),
	(ids::TRANSPOSE_UP, "&Up a Semitone\tCtrl+Shift+Up", "Transpose up by one semitone"),
	(ids::TRANSPOSE_RESET, "&Reset Transpose\tCtrl+Shift+0", "Play at the written pitch"),
];

const DEVICE_ITEMS: &[(i32, &str, &str)] =
	&[(ids::REFRESH_DEVICES, "&Refresh Output Devices\tF5", "Look for MIDI outputs again")];

const HELP_ITEMS: &[(i32, &str, &str)] = &[(ids::ABOUT, "&About Rumpus", "Version and license information")];

fn menu_builder(items: &[(i32, &str, &str)]) -> wxdragon::menus::menu::MenuBuilder {
	let mut builder = Menu::builder();
	for &(id, label, help) in items {
		builder = builder.append_item(id, label, help);
	}
	builder
}

fn menu(items: &[(i32, &str, &str)]) -> Menu {
	menu_builder(items).build()
}

#[must_use]
pub fn create_menu_bar() -> MenuBar {
	let file =
		menu_builder(FILE_ITEMS).append_separator().append_item(ids::EXIT, "E&xit\tCtrl+Q", "Close Rumpus").build();
	MenuBar::builder()
		.append(file, "&File")
		.append(menu(PLAYBACK_ITEMS), "&Playback")
		.append(menu(TEMPO_ITEMS), "&Tempo")
		.append(menu(TRANSPOSE_ITEMS), "T&ranspose")
		.append(menu(DEVICE_ITEMS), "&Device")
		.append(menu(HELP_ITEMS), "&Help")
		.build()
}

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use super::*;

	fn all_items() -> Vec<(i32, &'static str, &'static str)> {
		[FILE_ITEMS, PLAYBACK_ITEMS, TEMPO_ITEMS, TRANSPOSE_ITEMS, DEVICE_ITEMS, HELP_ITEMS].concat()
	}

	#[test]
	fn menu_ids_are_unique() {
		let ids: Vec<i32> = all_items().iter().map(|item| item.0).collect();
		let unique: HashSet<i32> = ids.iter().copied().collect();
		assert_eq!(ids.len(), unique.len());
	}

	#[test]
	fn shortcuts_are_unique() {
		let shortcuts: Vec<&str> = all_items().iter().filter_map(|item| item.1.split('\t').nth(1)).collect();
		let unique: HashSet<&str> = shortcuts.iter().copied().collect();
		assert_eq!(shortcuts.len(), unique.len(), "{shortcuts:?}");
	}

	#[test]
	fn every_playback_command_has_a_menu_item() {
		let ids: HashSet<i32> = all_items().iter().map(|item| item.0).collect();
		assert!(ids::PLAYBACK.iter().all(|id| ids.contains(id)));
	}
}
