//! File pickers and message boxes.

use std::path::PathBuf;

use wxdragon::prelude::*;

const ERROR_TITLE: &str = "Rumpus";

/// Modal, so focus moves into the dialog and screen readers read the message.
pub fn show_error(parent: &dyn WxWidget, message: &str) {
	wx_utils::show_error(parent, message, ERROR_TITLE);
}

pub fn pick_midi_files(parent: &dyn WxWidget) -> Vec<PathBuf> {
	let dialog = FileDialog::builder(parent)
		.with_message("Open MIDI files")
		.with_wildcard("MIDI files (*.mid;*.midi)|*.mid;*.midi|All files (*.*)|*.*")
		.with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist | FileDialogStyle::Multiple)
		.build();
	if dialog.show_modal() == ID_OK { dialog.get_paths().into_iter().map(PathBuf::from).collect() } else { Vec::new() }
}

pub fn pick_folder(parent: &dyn WxWidget) -> Option<PathBuf> {
	let dialog = DirDialog::builder(parent, "Add a folder of MIDI files", "").build();
	if dialog.show_modal() == ID_OK { dialog.get_path().map(PathBuf::from) } else { None }
}

pub fn show_about(parent: &Frame) {
	wx_utils::AboutBoxBuilder::new(parent)
		.name("Rumpus")
		.version(env!("CARGO_PKG_VERSION"))
		.description("An accessible MIDI file player for Windows MIDI Services")
		.copyright("Copyright (C) 2026 Leonard de Ruijter")
		.website(env!("CARGO_PKG_REPOSITORY"))
		.add_developer("Leonard de Ruijter")
		.show();
}
