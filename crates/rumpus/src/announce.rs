//! Screen reader announcements through a hidden live region.

use wxdragon::prelude::*;

/// Creates the hidden label that carries announcements; keep the handle and pass it to
/// [`announce`].
pub fn create_live_region(parent: Panel) -> StaticText {
	let label = StaticText::builder(&parent).with_label("").with_size(Size::new(0, 0)).build();
	label.show(false);
	if !live_region::set_live_region(&label) {
		tracing::warn!("could not register the live region; announcements are off");
	}
	label
}

pub fn announce(label: StaticText, message: &str) {
	tracing::debug!(message, "announce");
	live_region::announce(label, message);
}

/// Strips the mnemonic marker and trailing colon from a visible label for use as an
/// accessible name.
#[must_use]
pub fn accessible_name(label: &str) -> String {
	label.replace('&', "").trim_end_matches(':').trim().to_owned()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn accessible_name_drops_mnemonic_and_colon() {
		assert_eq!(accessible_name("&Output device:"), "Output device");
		assert_eq!(accessible_name("Tempo (%):"), "Tempo (%)");
	}
}
