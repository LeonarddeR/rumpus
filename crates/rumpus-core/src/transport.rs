//! Playback state machine: schedules song events ahead of a clock into a sink.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
	song::{ChannelMessage, EventKind, Song},
	ump::{channel_voice, panic_words, sysex7},
};

/// How far ahead of the clock events are handed to the sink.
pub const WINDOW_US: u64 = 500_000;
/// How long to wait between pumps while playing.
pub const PUMP_INTERVAL_US: u64 = 50_000;
/// Gap between the last scheduled event and the silence that follows it, and between that
/// silence and anything scheduled after it, so ordering does not depend on equal timestamps.
pub const SILENCE_MARGIN_US: u64 = 1_000;
pub const MIN_RATE: f64 = 0.25;
pub const MAX_RATE: f64 = 4.0;
pub const MAX_TRANSPOSE: i8 = 12;

const DRUM_CHANNEL: u8 = 9;
const FIRST_MODE_CONTROLLER: u8 = 120;
const BANK_SELECT_MSB: u8 = 0;
const BANK_SELECT_LSB: u8 = 32;

/// A monotonic clock in microseconds, shared with the sink's timestamps.
pub trait Clock {
	fn now_us(&self) -> u64;
}

/// Delivers UMP words to be played at an absolute clock time.
pub trait Sink {
	/// Fails when the sink cannot accept the message now; the transport retries later.
	fn send(&mut self, at_us: u64, words: &[u32]) -> Result<(), SendError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
	Stopped,
	Playing,
	Paused,
	/// Silence has been scheduled; the state after it lands is pending.
	Draining,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PumpOutcome {
	/// When the caller should pump again; `None` while nothing is scheduled.
	pub next_wake_us: Option<u64>,
	pub song_ended: bool,
}

/// Maps song time to wall time from a fixed point at a fixed rate.
#[derive(Clone, Copy, Debug)]
struct Anchor {
	wall_us: u64,
	song_us: u64,
}

impl Anchor {
	#[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	fn song_at(self, wall_us: u64, rate: f64) -> u64 {
		self.song_us + (wall_us.saturating_sub(self.wall_us) as f64 * rate).round() as u64
	}

	#[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	fn wall_at(self, song_us: u64, rate: f64) -> u64 {
		self.wall_us + (song_us.saturating_sub(self.song_us) as f64 / rate).round() as u64
	}
}

/// Per-channel state replayed after a seek so the synth is set up as if it had played through.
#[derive(Default)]
struct ChannelState {
	program: Option<u8>,
	controllers: BTreeMap<u8, u8>,
	pitch_bend: Option<u16>,
	pressure: Option<u8>,
}

impl ChannelState {
	fn apply(&mut self, message: ChannelMessage) {
		match message {
			ChannelMessage::ProgramChange { program } => self.program = Some(program),
			ChannelMessage::ControlChange { controller, value } if controller < FIRST_MODE_CONTROLLER => {
				self.controllers.insert(controller, value);
			}
			ChannelMessage::PitchBend { value } => self.pitch_bend = Some(value),
			ChannelMessage::ChannelPressure { pressure } => self.pressure = Some(pressure),
			_ => {}
		}
	}

	/// Bank select first, then the program, then everything else.
	fn messages(&self) -> Vec<ChannelMessage> {
		let bank = [BANK_SELECT_MSB, BANK_SELECT_LSB].into_iter().filter_map(|c| {
			self.controllers.get(&c).map(|&value| ChannelMessage::ControlChange { controller: c, value })
		});
		let program = self.program.map(|program| ChannelMessage::ProgramChange { program });
		let others = self
			.controllers
			.iter()
			.filter(|(c, _)| **c != BANK_SELECT_MSB && **c != BANK_SELECT_LSB)
			.map(|(&controller, &value)| ChannelMessage::ControlChange { controller, value });
		let bend = self.pitch_bend.map(|value| ChannelMessage::PitchBend { value });
		let pressure = self.pressure.map(|pressure| ChannelMessage::ChannelPressure { pressure });
		bank.chain(program).chain(others).chain(bend).chain(pressure).collect()
	}
}

pub struct Transport<C: Clock, S: Sink> {
	clock: C,
	sink: S,
	group: u8,
	song: Song,
	channels_used: BTreeSet<u8>,
	state: State,
	after_drain: State,
	drain_until_us: u64,
	rate: f64,
	transpose: i8,
	/// Song position while not playing.
	position_us: u64,
	/// Maps wall time to song time for the position readout.
	position_anchor: Anchor,
	/// Maps song time to wall time for scheduling; diverges from `position_anchor` after a rate
	/// change because already scheduled events keep their times.
	feed_anchor: Anchor,
	/// Index of the next event to schedule.
	cursor: usize,
	last_sent_us: u64,
	/// Sounding notes: (channel, key as written) to the key actually sent.
	sounding: BTreeMap<(u8, u8), u8>,
}

impl<C: Clock, S: Sink> Transport<C, S> {
	pub fn new(clock: C, sink: S, group: u8) -> Self {
		let origin = Anchor { wall_us: 0, song_us: 0 };
		Self {
			clock,
			sink,
			group,
			song: Song::default(),
			channels_used: BTreeSet::new(),
			state: State::Stopped,
			after_drain: State::Stopped,
			drain_until_us: 0,
			rate: 1.0,
			transpose: 0,
			position_us: 0,
			position_anchor: origin,
			feed_anchor: origin,
			cursor: 0,
			last_sent_us: 0,
			sounding: BTreeMap::new(),
		}
	}

	pub const fn sink(&self) -> &S {
		&self.sink
	}

	pub const fn sink_mut(&mut self) -> &mut S {
		&mut self.sink
	}

	pub const fn state(&self) -> State {
		self.state
	}

	pub const fn rate(&self) -> f64 {
		self.rate
	}

	pub const fn transpose(&self) -> i8 {
		self.transpose
	}

	pub const fn duration_us(&self) -> u64 {
		self.song.duration_us
	}

	/// Replaces the song, silencing anything still playing, and rewinds to the start.
	pub fn load(&mut self, mut song: Song) {
		if matches!(self.state, State::Playing | State::Draining) {
			self.silence(self.clock.now_us());
		}
		song.events.sort_by_key(|e| e.at_us);
		self.channels_used = song
			.events
			.iter()
			.filter_map(|e| match e.kind {
				EventKind::Channel { channel, .. } => Some(channel),
				EventKind::SysEx(_) => None,
			})
			.collect();
		self.song = song;
		self.state = State::Stopped;
		self.position_us = 0;
		self.cursor = 0;
		self.sounding.clear();
	}

	pub fn play(&mut self) {
		let now = self.clock.now_us();
		let start_wall = match self.state {
			State::Playing => return,
			State::Draining => now.max(self.drain_until_us + SILENCE_MARGIN_US),
			State::Stopped | State::Paused => now,
		};
		self.start_at(start_wall, self.position_us);
	}

	pub fn pause(&mut self) {
		if self.state == State::Playing {
			let silence_at = self.silence(self.clock.now_us());
			self.position_us = self.position_anchor.song_at(silence_at, self.rate).min(self.song.duration_us);
			self.drain(silence_at, State::Paused);
		}
	}

	pub fn stop(&mut self) {
		match self.state {
			State::Playing => {
				let silence_at = self.silence(self.clock.now_us());
				self.position_us = 0;
				self.drain(silence_at, State::Stopped);
			}
			State::Draining => {
				self.position_us = 0;
				self.after_drain = State::Stopped;
			}
			State::Paused => {
				self.position_us = 0;
				self.state = State::Stopped;
			}
			State::Stopped => {}
		}
	}

	pub fn seek_to(&mut self, song_us: u64) {
		let target = song_us.min(self.song.duration_us);
		if self.state == State::Playing {
			let silence_at = self.silence(self.clock.now_us());
			self.start_at(silence_at + SILENCE_MARGIN_US, target);
		} else {
			self.position_us = target;
		}
	}

	pub fn seek_by(&mut self, delta_us: i64) {
		let target = self.position_us().saturating_add_signed(delta_us);
		self.seek_to(target);
	}

	pub fn set_rate(&mut self, rate: f64) {
		let rate = rate.clamp(MIN_RATE, MAX_RATE);
		if self.state == State::Playing {
			let now = self.clock.now_us();
			let feed_song = self.feed_anchor.song_at(now + WINDOW_US, self.rate);
			self.position_anchor = Anchor { wall_us: now, song_us: self.position_anchor.song_at(now, self.rate) };
			self.feed_anchor = Anchor { wall_us: self.feed_anchor.wall_at(feed_song, self.rate), song_us: feed_song };
		}
		self.rate = rate;
	}

	pub fn set_transpose(&mut self, semitones: i8) {
		self.transpose = semitones.clamp(-MAX_TRANSPOSE, MAX_TRANSPOSE);
	}

	pub fn position_us(&self) -> u64 {
		match self.state {
			State::Playing => self.position_anchor.song_at(self.clock.now_us(), self.rate).min(self.song.duration_us),
			State::Paused | State::Stopped | State::Draining => self.position_us,
		}
	}

	/// Schedules due events and advances draining; call it again by the returned wake time.
	pub fn pump(&mut self) -> PumpOutcome {
		let now = self.clock.now_us();
		match self.state {
			State::Stopped | State::Paused => PumpOutcome::default(),
			State::Draining => {
				if now >= self.drain_until_us {
					self.state = self.after_drain;
					PumpOutcome::default()
				} else {
					PumpOutcome { next_wake_us: Some(self.drain_until_us), song_ended: false }
				}
			}
			State::Playing => self.feed(now),
		}
	}

	fn feed(&mut self, now: u64) -> PumpOutcome {
		let target_song = self.feed_anchor.song_at(now + WINDOW_US, self.rate);
		while let Some(event) = self.song.events.get(self.cursor).filter(|e| e.at_us <= target_song) {
			let at = self.feed_anchor.wall_at(event.at_us, self.rate);
			let kind = event.kind.clone();
			if self.send_event(at, &kind).is_err() {
				return PumpOutcome { next_wake_us: Some(now + PUMP_INTERVAL_US), song_ended: false };
			}
			self.cursor += 1;
			self.last_sent_us = self.last_sent_us.max(at);
		}
		let end_wall = self.feed_anchor.wall_at(self.song.duration_us, self.rate);
		let all_scheduled = self.cursor == self.song.events.len();
		if all_scheduled && now >= end_wall {
			self.state = State::Stopped;
			self.position_us = 0;
			self.sounding.clear();
			return PumpOutcome { next_wake_us: None, song_ended: true };
		}
		let next = if all_scheduled { end_wall.min(now + PUMP_INTERVAL_US) } else { now + PUMP_INTERVAL_US };
		PumpOutcome { next_wake_us: Some(next), song_ended: false }
	}

	/// Chases channel state up to `song_us`, then plays on from there starting at `wall_us`.
	fn start_at(&mut self, wall_us: u64, song_us: u64) {
		let mut channels: BTreeMap<u8, ChannelState> = BTreeMap::new();
		self.cursor = self.song.events.partition_point(|e| e.at_us < song_us);
		for event in &self.song.events[..self.cursor] {
			if let EventKind::Channel { channel, message } = event.kind {
				channels.entry(channel).or_default().apply(message);
			}
		}
		for (channel, state) in &channels {
			for message in state.messages() {
				let _ = self.sink.send(wall_us, &[channel_voice(self.group, *channel, message)]);
			}
		}
		let anchor = Anchor { wall_us, song_us };
		self.position_anchor = anchor;
		self.feed_anchor = anchor;
		self.last_sent_us = self.last_sent_us.max(wall_us);
		self.state = State::Playing;
	}

	const fn drain(&mut self, until_us: u64, then: State) {
		self.state = State::Draining;
		self.drain_until_us = until_us;
		self.after_drain = then;
	}

	/// Schedules note-offs for sounding notes plus a panic on every used channel, after the last
	/// scheduled event, and returns when that lands.
	fn silence(&mut self, now: u64) -> u64 {
		let at = now.max(self.last_sent_us + SILENCE_MARGIN_US);
		for (&(channel, _), &sent_key) in &self.sounding {
			let word = channel_voice(self.group, channel, ChannelMessage::NoteOff { key: sent_key, velocity: 0 });
			let _ = self.sink.send(at, &[word]);
		}
		self.sounding.clear();
		for &channel in &self.channels_used {
			for word in panic_words(self.group, channel) {
				let _ = self.sink.send(at, &[word]);
			}
		}
		self.last_sent_us = at;
		at
	}

	fn send_event(&mut self, at: u64, kind: &EventKind) -> Result<(), SendError> {
		match kind {
			EventKind::Channel { channel, message } => match self.transposed(*channel, *message) {
				Some(message) => self.sink.send(at, &[channel_voice(self.group, *channel, message)]),
				None => Ok(()),
			},
			EventKind::SysEx(payload) => self.sink.send(at, &sysex7(self.group, payload)),
		}
	}

	/// Applies the transposition, remembering which key a note was sent as; `None` drops the
	/// message because the key left the MIDI range.
	fn transposed(&mut self, channel: u8, message: ChannelMessage) -> Option<ChannelMessage> {
		let shift = |key: u8| -> Option<u8> {
			if channel == DRUM_CHANNEL {
				Some(key)
			} else {
				u8::try_from(i16::from(key) + i16::from(self.transpose)).ok().filter(|k| *k <= 127)
			}
		};
		match message {
			ChannelMessage::NoteOn { key, velocity } => {
				let sent = shift(key)?;
				self.sounding.insert((channel, key), sent);
				Some(ChannelMessage::NoteOn { key: sent, velocity })
			}
			ChannelMessage::NoteOff { key, velocity } => {
				let sent = self.sounding.remove(&(channel, key)).or_else(|| shift(key))?;
				Some(ChannelMessage::NoteOff { key: sent, velocity })
			}
			ChannelMessage::PolyPressure { key, pressure } => {
				let sent = self.sounding.get(&(channel, key)).copied().or_else(|| shift(key))?;
				Some(ChannelMessage::PolyPressure { key: sent, pressure })
			}
			other => Some(other),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{cell::Cell, rc::Rc};

	use super::*;
	use crate::{
		song::{ChannelMessage, EventKind, Song, TimedEvent},
		ump::{channel_voice, panic_words},
	};

	const START: u64 = 1_000_000;

	#[derive(Clone, Default)]
	struct FakeClock(Rc<Cell<u64>>);

	impl Clock for FakeClock {
		fn now_us(&self) -> u64 {
			self.0.get()
		}
	}

	#[derive(Default)]
	struct RecordingSink {
		sent: Vec<(u64, Vec<u32>)>,
		failures_left: usize,
	}

	impl Sink for RecordingSink {
		fn send(&mut self, at_us: u64, words: &[u32]) -> Result<(), SendError> {
			if self.failures_left > 0 {
				self.failures_left -= 1;
				return Err(SendError);
			}
			self.sent.push((at_us, words.to_vec()));
			Ok(())
		}
	}

	type TestTransport = Transport<FakeClock, RecordingSink>;

	fn channel_event(at_us: u64, channel: u8, message: ChannelMessage) -> TimedEvent {
		TimedEvent { at_us, kind: EventKind::Channel { channel, message } }
	}

	fn note_on(at_us: u64, key: u8) -> TimedEvent {
		channel_event(at_us, 0, ChannelMessage::NoteOn { key, velocity: 100 })
	}

	fn note_off(at_us: u64, key: u8) -> TimedEvent {
		channel_event(at_us, 0, ChannelMessage::NoteOff { key, velocity: 0 })
	}

	fn song(events: Vec<TimedEvent>) -> Song {
		let duration_us = events.iter().map(|e| e.at_us).max().unwrap_or(0);
		Song { title: String::new(), duration_us, events }
	}

	fn transport(events: Vec<TimedEvent>) -> (TestTransport, FakeClock) {
		let clock = FakeClock::default();
		clock.0.set(START);
		let mut transport = Transport::new(clock.clone(), RecordingSink::default(), 0);
		transport.load(song(events));
		(transport, clock)
	}

	fn word(channel: u8, message: ChannelMessage) -> u32 {
		channel_voice(0, channel, message)
	}

	fn sent(transport: &TestTransport) -> Vec<(u64, u32)> {
		transport.sink().sent.iter().map(|(at, words)| (*at, words[0])).collect()
	}

	#[test]
	fn play_schedules_only_events_inside_the_window() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_on(400_000, 62), note_on(600_000, 64)]);
		t.play();
		t.pump();
		assert_eq!(
			sent(&t),
			[
				(START, word(0, ChannelMessage::NoteOn { key: 60, velocity: 100 })),
				(START + 400_000, word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 }))
			]
		);
		clock.0.set(START + 200_000);
		t.pump();
		assert_eq!(
			sent(&t).last(),
			Some(&(START + 600_000, word(0, ChannelMessage::NoteOn { key: 64, velocity: 100 })))
		);
	}

	#[test]
	fn rate_scales_event_times() {
		let (mut t, _clock) = transport(vec![note_on(400_000, 60)]);
		t.set_rate(2.0);
		t.play();
		t.pump();
		assert_eq!(sent(&t)[0].0, START + 200_000);
	}

	#[test]
	fn position_follows_the_clock_and_stays_continuous_across_rate_changes() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_off(5_000_000, 60)]);
		t.play();
		t.pump();
		clock.0.set(START + 200_000);
		assert_eq!(t.position_us(), 200_000);
		t.set_rate(2.0);
		assert_eq!(t.position_us(), 200_000);
		clock.0.set(START + 300_000);
		assert_eq!(t.position_us(), 400_000);
	}

	#[test]
	fn pause_silences_after_the_last_scheduled_event_and_keeps_the_position() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_off(2_000_000, 60)]);
		t.play();
		t.pump();
		t.pause();
		let silence_at = START + SILENCE_MARGIN_US;
		let sends = sent(&t);
		assert_eq!(
			&sends[1..],
			[
				(silence_at, word(0, ChannelMessage::NoteOff { key: 60, velocity: 0 })),
				(silence_at, panic_words(0, 0)[0]),
				(silence_at, panic_words(0, 0)[1]),
				(silence_at, panic_words(0, 0)[2]),
			]
		);
		assert_eq!(t.state(), State::Draining);
		clock.0.set(silence_at);
		t.pump();
		assert_eq!(t.state(), State::Paused);
		assert_eq!(t.position_us(), SILENCE_MARGIN_US);
	}

	#[test]
	fn stop_resets_the_position() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_off(2_000_000, 60)]);
		t.play();
		t.pump();
		clock.0.set(START + 300_000);
		t.stop();
		clock.0.set(START + 1_000_000);
		t.pump();
		assert_eq!(t.state(), State::Stopped);
		assert_eq!(t.position_us(), 0);
	}

	#[test]
	fn resuming_after_pause_continues_from_the_paused_position() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_on(800_000, 62), note_off(2_000_000, 60)]);
		t.play();
		t.pump();
		clock.0.set(START + 300_000);
		t.pause();
		clock.0.set(START + 2_000_000);
		t.pump();
		let before = sent(&t).len();
		t.play();
		t.pump();
		let resumed = &sent(&t)[before..];
		assert_eq!(
			resumed.last(),
			Some(&(START + 2_000_000 + 500_000, word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 })))
		);
	}

	#[test]
	fn transposed_note_off_matches_the_note_that_was_sent() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_off(700_000, 60)]);
		t.set_transpose(2);
		t.play();
		t.pump();
		t.set_transpose(-5);
		clock.0.set(START + 300_000);
		t.pump();
		assert_eq!(
			sent(&t),
			[
				(START, word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 })),
				(START + 700_000, word(0, ChannelMessage::NoteOff { key: 62, velocity: 0 })),
			]
		);
	}

	#[test]
	fn channel_10_is_never_transposed() {
		let (mut t, _clock) = transport(vec![channel_event(0, 9, ChannelMessage::NoteOn { key: 36, velocity: 100 })]);
		t.set_transpose(12);
		t.play();
		t.pump();
		assert_eq!(sent(&t)[0].1, word(9, ChannelMessage::NoteOn { key: 36, velocity: 100 }));
	}

	#[test]
	fn notes_transposed_out_of_range_are_dropped() {
		let (mut t, _clock) = transport(vec![note_on(0, 127), note_off(100_000, 127)]);
		t.set_transpose(1);
		t.play();
		t.pump();
		assert!(sent(&t).is_empty());
	}

	#[test]
	fn play_from_a_position_chases_bank_program_controllers_and_pitch_bend() {
		let (mut t, _clock) = transport(vec![
			channel_event(0, 0, ChannelMessage::ControlChange { controller: 0, value: 1 }),
			channel_event(0, 0, ChannelMessage::ProgramChange { program: 5 }),
			channel_event(100_000, 0, ChannelMessage::ControlChange { controller: 7, value: 100 }),
			channel_event(200_000, 0, ChannelMessage::PitchBend { value: 9000 }),
			note_on(300_000, 60),
			note_off(400_000, 60),
			channel_event(900_000, 0, ChannelMessage::ControlChange { controller: 7, value: 50 }),
			channel_event(950_000, 0, ChannelMessage::ControlChange { controller: 123, value: 0 }),
			note_on(1_200_000, 62),
		]);
		t.seek_to(1_000_000);
		t.play();
		t.pump();
		assert_eq!(
			sent(&t),
			[
				(START, word(0, ChannelMessage::ControlChange { controller: 0, value: 1 })),
				(START, word(0, ChannelMessage::ProgramChange { program: 5 })),
				(START, word(0, ChannelMessage::ControlChange { controller: 7, value: 50 })),
				(START, word(0, ChannelMessage::PitchBend { value: 9000 })),
				(START + 200_000, word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 })),
			]
		);
	}

	#[test]
	fn seeking_while_playing_silences_then_resumes_after_the_silence() {
		let (mut t, _clock) = transport(vec![note_on(0, 60), note_off(3_000_000, 60), note_on(2_000_000, 62)]);
		t.play();
		t.pump();
		t.seek_to(1_900_000);
		t.pump();
		let sends = sent(&t);
		let silence_at = START + SILENCE_MARGIN_US;
		assert_eq!(sends[1], (silence_at, word(0, ChannelMessage::NoteOff { key: 60, velocity: 0 })));
		assert_eq!(
			sends.last(),
			Some(&(
				silence_at + SILENCE_MARGIN_US + 100_000,
				word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 })
			))
		);
		assert_eq!(t.state(), State::Playing);
	}

	#[test]
	fn seek_by_clamps_to_the_song() {
		let (mut t, _clock) = transport(vec![note_on(0, 60), note_off(2_000_000, 60)]);
		t.seek_by(-5_000_000);
		assert_eq!(t.position_us(), 0);
		t.seek_by(5_000_000);
		assert_eq!(t.position_us(), 2_000_000);
	}

	#[test]
	fn a_failed_send_stops_the_pump_and_is_retried_in_order() {
		let (mut t, _clock) = transport(vec![note_on(0, 60), note_on(100_000, 62)]);
		t.sink_mut().failures_left = 1;
		t.play();
		assert_eq!(t.pump().next_wake_us, Some(START + PUMP_INTERVAL_US));
		assert!(sent(&t).is_empty());
		t.pump();
		assert_eq!(
			sent(&t),
			[
				(START, word(0, ChannelMessage::NoteOn { key: 60, velocity: 100 })),
				(START + 100_000, word(0, ChannelMessage::NoteOn { key: 62, velocity: 100 })),
			]
		);
	}

	#[test]
	fn the_song_ends_when_its_duration_has_passed() {
		let (mut t, clock) = transport(vec![note_on(0, 60), note_off(300_000, 60)]);
		t.play();
		assert!(!t.pump().song_ended);
		clock.0.set(START + 299_000);
		assert!(!t.pump().song_ended);
		clock.0.set(START + 300_000);
		assert!(t.pump().song_ended);
		assert_eq!(t.state(), State::Stopped);
		assert_eq!(t.position_us(), 0);
	}

	#[test]
	fn pump_reports_the_next_wake_time() {
		let (mut t, _clock) = transport(vec![note_on(0, 60), note_off(3_000_000, 60)]);
		assert_eq!(t.pump().next_wake_us, None);
		t.play();
		assert_eq!(t.pump().next_wake_us, Some(START + PUMP_INTERVAL_US));
	}
}
