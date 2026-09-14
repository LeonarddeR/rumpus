//! Tiny Standard MIDI Files for tests, so no real songs are needed.

use std::path::{Path, PathBuf};

use midly::{
	Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
	num::{u4, u7, u15, u28},
};

const TICKS_PER_BEAT: u16 = 480;

/// Writes `three notes.mid`: C, D and E on channel 1, one beat each, one and a half seconds at
/// the default tempo.
#[must_use]
pub fn write_three_notes(dir: &Path) -> PathBuf {
	let note = |delta: u32, key: u8, vel: u8| TrackEvent {
		delta: u28::new(delta),
		kind: TrackEventKind::Midi {
			channel: u4::new(0),
			message: MidiMessage::NoteOn { key: u7::new(key), vel: u7::new(vel) },
		},
	};
	let track = vec![
		note(0, 60, 100),
		note(480, 60, 0),
		note(0, 62, 100),
		note(480, 62, 0),
		note(0, 64, 100),
		note(480, 64, 0),
		end_of_track(),
	];
	write(dir, "three notes.mid", track)
}

/// Writes `long note.mid`: one note on channel 3 that sounds for ten seconds.
#[must_use]
pub fn write_long_note(dir: &Path) -> PathBuf {
	let event = |delta: u32, message: MidiMessage| TrackEvent {
		delta: u28::new(delta),
		kind: TrackEventKind::Midi { channel: u4::new(2), message },
	};
	let track = vec![
		event(0, MidiMessage::NoteOn { key: u7::new(48), vel: u7::new(100) }),
		event(9600, MidiMessage::NoteOff { key: u7::new(48), vel: u7::new(0) }),
		end_of_track(),
	];
	write(dir, "long note.mid", track)
}

const fn end_of_track() -> TrackEvent<'static> {
	TrackEvent { delta: u28::new(0), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) }
}

fn write(dir: &Path, file_name: &str, track: Vec<TrackEvent<'_>>) -> PathBuf {
	std::fs::create_dir_all(dir).expect("create the fixture directory");
	let smf = Smf {
		header: Header::new(Format::SingleTrack, Timing::Metrical(u15::new(TICKS_PER_BEAT))),
		tracks: vec![track],
	};
	let path = dir.join(file_name);
	smf.save(&path).expect("write the fixture");
	path
}
