//! Human-readable durations for the position readout.

const US_PER_SECOND: u64 = 1_000_000;

/// Formats a duration as `m:ss`, or `h:mm:ss` from one hour on, truncating to whole seconds.
#[must_use]
pub fn mmss(us: u64) -> String {
	let total_seconds = us / US_PER_SECOND;
	let seconds = total_seconds % 60;
	let minutes = total_seconds / 60 % 60;
	let hours = total_seconds / 3600;
	if hours > 0 { format!("{hours}:{minutes:02}:{seconds:02}") } else { format!("{minutes}:{seconds:02}") }
}

/// Formats an elapsed and total duration as `m:ss / m:ss`.
#[must_use]
pub fn position_text(elapsed_us: u64, total_us: u64) -> String {
	format!("{} / {}", mmss(elapsed_us), mmss(total_us))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn zero_is_zero_minutes() {
		assert_eq!(mmss(0), "0:00");
	}

	#[test]
	fn seconds_are_zero_padded_and_truncated() {
		assert_eq!(mmss(7_999_999), "0:07");
		assert_eq!(mmss(187_000_000), "3:07");
	}

	#[test]
	fn hours_appear_when_needed() {
		assert_eq!(mmss(3_723_000_000), "1:02:03");
	}

	#[test]
	fn position_text_shows_elapsed_and_total() {
		assert_eq!(position_text(83_000_000, 296_000_000), "1:23 / 4:56");
	}
}
