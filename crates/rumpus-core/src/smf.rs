//! Standard MIDI File loading into a [`Song`].

use std::{io, path::Path};

use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

use crate::song::{ChannelMessage, EventKind, Song, TimedEvent};

const DEFAULT_US_PER_BEAT: u64 = 500_000;
const SYSEX_END: u8 = 0xF7;

#[derive(Debug, thiserror::Error)]
pub enum SmfError {
	#[error("cannot read file: {0}")]
	Io(#[from] io::Error),
	#[error("not a valid MIDI file: {0}")]
	Format(#[from] midly::Error),
}

/// Loads a Standard MIDI File; the file stem is the title when the file names none.
pub fn load(path: &Path) -> Result<Song, SmfError> {
	let bytes = std::fs::read(path)?;
	let mut song = parse(&bytes)?;
	if song.title.is_empty() {
		song.title = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
	}
	Ok(song)
}

/// Parses SMF bytes into a song whose events carry absolute microsecond times at 100% tempo.
pub fn parse(bytes: &[u8]) -> Result<Song, SmfError> {
	let smf = Smf::parse(bytes)?;
	let mut title = String::new();
	let mut raw = Vec::new();
	let mut last_tick = 0u64;
	for track in &smf.tracks {
		let mut tick = 0u64;
		let mut pending_sysex: Option<Vec<u8>> = None;
		for event in track {
			tick += u64::from(event.delta.as_int());
			last_tick = last_tick.max(tick);
			match event.kind {
				TrackEventKind::Meta(MetaMessage::TrackName(name)) if title.is_empty() => {
					String::from_utf8_lossy(name).trim().clone_into(&mut title);
				}
				TrackEventKind::Meta(MetaMessage::Tempo(us_per_beat)) => {
					raw.push((tick, RawEvent::Tempo(u64::from(us_per_beat.as_int()))));
				}
				TrackEventKind::Midi { channel, message } => {
					raw.push((tick, RawEvent::Channel { channel: channel.as_int(), message: convert(message) }));
				}
				TrackEventKind::SysEx(data) => {
					pending_sysex = push_sysex_chunk(&mut raw, tick, Vec::new(), data);
				}
				// Escape events outside a split system exclusive message are dropped.
				TrackEventKind::Escape(data) => {
					if let Some(buffer) = pending_sysex.take() {
						pending_sysex = push_sysex_chunk(&mut raw, tick, buffer, data);
					}
				}
				TrackEventKind::Meta(_) => {}
			}
		}
	}
	// Sorting by tick alone keeps track order and file order for equal ticks.
	raw.sort_by_key(|&(tick, ..)| tick);

	let mut clock = TickClock::new(smf.header.timing);
	let mut events = Vec::new();
	for (tick, raw_event) in raw {
		let at_us = clock.us_at(tick);
		match raw_event {
			RawEvent::Tempo(us_per_beat) => clock.set_tempo(tick, us_per_beat),
			RawEvent::Channel { channel, message } => {
				events.push(TimedEvent { at_us, kind: EventKind::Channel { channel, message } });
			}
			RawEvent::SysEx(payload) => events.push(TimedEvent { at_us, kind: EventKind::SysEx(payload) }),
		}
	}
	Ok(Song { title, duration_us: clock.us_at(last_tick), events })
}

/// A track event in tick order, before tempo is resolved.
enum RawEvent {
	Tempo(u64),
	Channel { channel: u8, message: ChannelMessage },
	SysEx(Vec<u8>),
}

/// Appends a system exclusive chunk to `buffer`; a chunk ending in `F7` completes the message
/// and pushes it, otherwise the buffer is returned to await the next chunk.
fn push_sysex_chunk(raw: &mut Vec<(u64, RawEvent)>, tick: u64, mut buffer: Vec<u8>, chunk: &[u8]) -> Option<Vec<u8>> {
	if let Some(payload) = chunk.strip_suffix(&[SYSEX_END]) {
		buffer.extend_from_slice(payload);
		raw.push((tick, RawEvent::SysEx(buffer)));
		None
	} else {
		buffer.extend_from_slice(chunk);
		Some(buffer)
	}
}

/// Converts a channel message, mapping note on with velocity zero to note off.
fn convert(message: MidiMessage) -> ChannelMessage {
	match message {
		MidiMessage::NoteOff { key, vel } => ChannelMessage::NoteOff { key: key.as_int(), velocity: vel.as_int() },
		MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => {
			ChannelMessage::NoteOff { key: key.as_int(), velocity: 0 }
		}
		MidiMessage::NoteOn { key, vel } => ChannelMessage::NoteOn { key: key.as_int(), velocity: vel.as_int() },
		MidiMessage::Aftertouch { key, vel } => {
			ChannelMessage::PolyPressure { key: key.as_int(), pressure: vel.as_int() }
		}
		MidiMessage::Controller { controller, value } => {
			ChannelMessage::ControlChange { controller: controller.as_int(), value: value.as_int() }
		}
		MidiMessage::ProgramChange { program } => ChannelMessage::ProgramChange { program: program.as_int() },
		MidiMessage::ChannelAftertouch { vel } => ChannelMessage::ChannelPressure { pressure: vel.as_int() },
		MidiMessage::PitchBend { bend } => ChannelMessage::PitchBend { value: bend.0.as_int() },
	}
}

/// Converts ticks to microseconds, following tempo changes in tick order.
struct TickClock {
	timing: Timing,
	us_per_beat: u64,
	anchor_tick: u64,
	anchor_us: u64,
}

impl TickClock {
	const fn new(timing: Timing) -> Self {
		Self { timing, us_per_beat: DEFAULT_US_PER_BEAT, anchor_tick: 0, anchor_us: 0 }
	}

	fn us_at(&self, tick: u64) -> u64 {
		let ticks = tick - self.anchor_tick;
		match self.timing {
			Timing::Metrical(ticks_per_beat) => {
				self.anchor_us + ticks * self.us_per_beat / u64::from(ticks_per_beat.as_int())
			}
			Timing::Timecode(fps, ticks_per_frame) => {
				tick * 1_000_000 / (u64::from(fps.as_int()) * u64::from(ticks_per_frame))
			}
		}
	}

	fn set_tempo(&mut self, tick: u64, us_per_beat: u64) {
		self.anchor_us = self.us_at(tick);
		self.anchor_tick = tick;
		self.us_per_beat = us_per_beat;
	}
}

#[cfg(test)]
mod tests {
	use midly::{
		Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
		num::{u4, u7, u15, u24, u28},
	};

	use super::*;
	use crate::song::{ChannelMessage, EventKind};

	const TICKS_PER_BEAT: u16 = 480;

	fn event(delta: u32, kind: TrackEventKind<'_>) -> TrackEvent<'_> {
		TrackEvent { delta: u28::new(delta), kind }
	}

	fn note_on(delta: u32, key: u8, velocity: u8) -> TrackEvent<'static> {
		event(
			delta,
			TrackEventKind::Midi {
				channel: u4::new(0),
				message: MidiMessage::NoteOn { key: u7::new(key), vel: u7::new(velocity) },
			},
		)
	}

	fn sysex(delta: u32, data: &[u8]) -> TrackEvent<'_> {
		event(delta, TrackEventKind::SysEx(data))
	}

	fn escape(delta: u32, data: &[u8]) -> TrackEvent<'_> {
		event(delta, TrackEventKind::Escape(data))
	}

	fn tempo(delta: u32, us_per_beat: u32) -> TrackEvent<'static> {
		event(delta, TrackEventKind::Meta(MetaMessage::Tempo(u24::new(us_per_beat))))
	}

	fn end_of_track(delta: u32) -> TrackEvent<'static> {
		event(delta, TrackEventKind::Meta(MetaMessage::EndOfTrack))
	}

	fn smf_bytes(format: Format, timing: Timing, tracks: Vec<Vec<TrackEvent<'_>>>) -> Vec<u8> {
		let smf = Smf { header: Header::new(format, timing), tracks };
		let mut bytes = Vec::new();
		smf.write_std(&mut bytes).expect("write SMF");
		bytes
	}

	fn metrical(tracks: Vec<Vec<TrackEvent<'_>>>) -> Vec<u8> {
		smf_bytes(Format::Parallel, Timing::Metrical(u15::new(TICKS_PER_BEAT)), tracks)
	}

	fn times(song: &Song) -> Vec<u64> {
		song.events.iter().map(|e| e.at_us).collect()
	}

	#[test]
	fn default_tempo_is_120_bpm() {
		let bytes = metrical(vec![vec![note_on(0, 60, 100), note_on(480, 60, 0), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(times(&song), [0, 500_000]);
	}

	#[test]
	fn tempo_changes_apply_from_their_tick_onwards() {
		let bytes = metrical(vec![vec![
			tempo(0, 500_000),
			note_on(480, 60, 100),
			tempo(0, 250_000),
			note_on(480, 62, 100),
			end_of_track(0),
		]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(times(&song), [500_000, 750_000]);
	}

	#[test]
	fn tempo_track_governs_other_tracks_in_format_1() {
		let bytes =
			metrical(vec![vec![tempo(0, 250_000), end_of_track(0)], vec![note_on(480, 60, 100), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(times(&song), [250_000]);
	}

	#[test]
	fn events_at_the_same_tick_keep_track_order() {
		let bytes =
			metrical(vec![vec![note_on(10, 60, 100), end_of_track(0)], vec![note_on(10, 62, 100), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		let keys: Vec<u8> = song
			.events
			.iter()
			.map(|e| match e.kind {
				EventKind::Channel { message: ChannelMessage::NoteOn { key, .. }, .. } => key,
				_ => panic!("unexpected event"),
			})
			.collect();
		assert_eq!(keys, [60, 62]);
	}

	#[test]
	fn note_on_with_zero_velocity_becomes_note_off() {
		let bytes = metrical(vec![vec![note_on(0, 60, 0), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(
			song.events[0].kind,
			EventKind::Channel { channel: 0, message: ChannelMessage::NoteOff { key: 60, velocity: 0 } }
		);
	}

	#[test]
	fn sysex_payload_drops_the_trailing_end_byte() {
		let bytes =
			metrical(vec![vec![event(0, TrackEventKind::SysEx(&[0x7E, 0x7F, 0x09, 0x01, 0xF7])), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(song.events[0].kind, EventKind::SysEx(vec![0x7E, 0x7F, 0x09, 0x01]));
	}

	#[test]
	fn split_sysex_is_reassembled_at_the_final_chunk() {
		let bytes = metrical(vec![vec![
			sysex(0, &[0x7E, 0x7F]),
			note_on(480, 60, 100),
			escape(0, &[0x09]),
			escape(480, &[0x01, 0xF7]),
			end_of_track(0),
		]]);
		let song = parse(&bytes).unwrap();
		let sysex: Vec<_> = song.events.iter().filter(|e| matches!(e.kind, EventKind::SysEx(_))).collect();
		assert_eq!(sysex.len(), 1);
		assert_eq!(sysex[0].kind, EventKind::SysEx(vec![0x7E, 0x7F, 0x09, 0x01]));
		assert_eq!(sysex[0].at_us, 1_000_000);
	}

	#[test]
	fn escape_without_a_pending_sysex_is_dropped() {
		let bytes = metrical(vec![vec![escape(0, &[0xF3, 0x05]), end_of_track(0)]]);
		let song = parse(&bytes).unwrap();
		assert!(song.events.is_empty());
	}

	#[test]
	fn unterminated_sysex_is_dropped_at_the_end_of_its_track() {
		let bytes = metrical(vec![
			vec![sysex(0, &[0x7E, 0x7F]), end_of_track(0)],
			vec![escape(0, &[0x09, 0x01, 0xF7]), end_of_track(0)],
		]);
		let song = parse(&bytes).unwrap();
		assert!(song.events.is_empty());
	}

	#[test]
	fn title_comes_from_the_first_track_name() {
		let bytes = metrical(vec![
			vec![event(0, TrackEventKind::Meta(MetaMessage::TrackName(b"My Song"))), end_of_track(0)],
			vec![event(0, TrackEventKind::Meta(MetaMessage::TrackName(b"Piano"))), end_of_track(0)],
		]);
		let song = parse(&bytes).unwrap();
		assert_eq!(song.title, "My Song");
	}

	#[test]
	fn title_is_empty_without_a_track_name() {
		let bytes = metrical(vec![vec![end_of_track(0)]]);
		assert_eq!(parse(&bytes).unwrap().title, "");
	}

	#[test]
	fn duration_is_the_latest_end_of_track() {
		let bytes = metrical(vec![vec![note_on(0, 60, 100), end_of_track(480)], vec![end_of_track(960)]]);
		let song = parse(&bytes).unwrap();
		assert_eq!(song.duration_us, 1_000_000);
	}

	#[test]
	fn timecode_timing_uses_frames_and_subframes() {
		let bytes = smf_bytes(
			Format::SingleTrack,
			Timing::Timecode(midly::Fps::Fps25, 40),
			vec![vec![note_on(500, 60, 100), end_of_track(0)]],
		);
		let song = parse(&bytes).unwrap();
		assert_eq!(times(&song), [500_000]);
	}

	#[test]
	fn garbage_is_an_error() {
		assert!(parse(b"not a midi file").is_err());
	}
}
