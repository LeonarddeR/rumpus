//! A parsed MIDI file as a flat, time-resolved list of events.

/// A MIDI 1.0 channel voice message with its data bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelMessage {
	NoteOff {
		key: u8,
		velocity: u8,
	},
	NoteOn {
		key: u8,
		velocity: u8,
	},
	PolyPressure {
		key: u8,
		pressure: u8,
	},
	ControlChange {
		controller: u8,
		value: u8,
	},
	ProgramChange {
		program: u8,
	},
	ChannelPressure {
		pressure: u8,
	},
	/// 14-bit value, 0 to 16383, centre 8192.
	PitchBend {
		value: u16,
	},
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
	Channel {
		channel: u8,
		message: ChannelMessage,
	},
	/// System exclusive payload without the leading `F0` and trailing `F7`.
	SysEx(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimedEvent {
	/// Absolute time from the start of the song at 100% tempo.
	pub at_us: u64,
	pub kind: EventKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Song {
	pub title: String,
	pub duration_us: u64,
	/// Events sorted by `at_us`, stable with respect to file order.
	pub events: Vec<TimedEvent>,
}
