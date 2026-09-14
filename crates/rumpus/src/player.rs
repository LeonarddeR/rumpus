//! The playback thread: owns the MIDI session and transport, driven by commands from the UI.

use std::{
	path::{Path, PathBuf},
	sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
	thread::{self, JoinHandle},
	time::{Duration, Instant},
};

use rumpus_core::{
	config::OutputSelection,
	smf,
	song::Song,
	transport::{Clock, State, Transport},
};

use crate::midi::backend::{self, Connection, OutputDevice, ServiceClock, Session};

const SESSION_NAME: &str = "Rumpus";
const IDLE_TIMEOUT: Duration = Duration::from_millis(250);
const POSITION_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
	Probe,
	RefreshOutputs,
	SelectOutput(OutputSelection),
	Load(PathBuf),
	Play,
	Pause,
	Stop,
	SeekBy(i64),
	SetRate(f64),
	SetTranspose(i8),
	Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayerEvent {
	Unavailable(String),
	Outputs(Vec<OutputDevice>),
	Loaded { title: String, duration_us: u64 },
	State(State),
	Position(u64),
	SongEnded,
	Error(String),
}

/// Handle to the playback thread; dropping it shuts the thread down.
pub struct Player {
	tx: Sender<Command>,
	thread: Option<JoinHandle<()>>,
}

impl Player {
	pub fn send(&self, command: Command) {
		let _ = self.tx.send(command);
	}

	pub fn shutdown(mut self) {
		self.join();
	}

	fn join(&mut self) {
		self.send(Command::Shutdown);
		if let Some(thread) = self.thread.take() {
			let _ = thread.join();
		}
	}
}

impl Drop for Player {
	fn drop(&mut self) {
		self.join();
	}
}

/// Starts the playback thread; `on_event` is called on that thread for every event.
pub fn spawn(on_event: impl Fn(PlayerEvent) + Send + 'static) -> Player {
	let (tx, rx) = mpsc::channel();
	let thread = thread::Builder::new()
		.name("rumpus-player".to_owned())
		.spawn(move || run(&rx, &on_event))
		.expect("spawn the player thread");
	Player { tx, thread: Some(thread) }
}

fn run(rx: &Receiver<Command>, on_event: &dyn Fn(PlayerEvent)) {
	if let Err(e) = backend::init() {
		on_event(PlayerEvent::Unavailable(format!("Could not initialize the Windows Runtime: {e}")));
		return;
	}
	let mut worker = Worker::new(on_event);
	loop {
		match rx.recv_timeout(worker.timeout()) {
			Ok(Command::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
			Ok(command) => worker.handle(command),
			Err(RecvTimeoutError::Timeout) => {}
		}
		worker.pump();
	}
	worker.silence();
}

struct Worker<'a> {
	on_event: &'a dyn Fn(PlayerEvent),
	session: Option<Session>,
	transport: Option<Transport<ServiceClock, Connection>>,
	song: Option<Song>,
	rate: f64,
	transpose: i8,
	reported_state: State,
	last_position_report: Instant,
	next_wake_us: Option<u64>,
}

impl<'a> Worker<'a> {
	fn new(on_event: &'a dyn Fn(PlayerEvent)) -> Self {
		Self {
			on_event,
			session: None,
			transport: None,
			song: None,
			rate: 1.0,
			transpose: 0,
			reported_state: State::Stopped,
			last_position_report: Instant::now(),
			next_wake_us: None,
		}
	}

	fn emit(&self, event: PlayerEvent) {
		(self.on_event)(event);
	}

	fn timeout(&self) -> Duration {
		self.next_wake_us.map_or(IDLE_TIMEOUT, |wake| {
			Duration::from_micros(wake.saturating_sub(ServiceClock.now_us())).min(IDLE_TIMEOUT)
		})
	}

	fn handle(&mut self, command: Command) {
		match command {
			Command::Probe => match backend::probe() {
				Ok(()) => self.emit(PlayerEvent::Outputs(backend::outputs(false))),
				Err(e) => self.emit(PlayerEvent::Unavailable(e.to_string())),
			},
			Command::RefreshOutputs => self.emit(PlayerEvent::Outputs(backend::outputs(false))),
			Command::SelectOutput(output) => self.select_output(&output),
			Command::Load(path) => self.load(&path),
			Command::Play => self.with_transport(Transport::play),
			Command::Pause => self.with_transport(Transport::pause),
			Command::Stop => self.with_transport(Transport::stop),
			Command::SeekBy(delta_us) => {
				self.with_transport(|t| t.seek_by(delta_us));
				self.report_position();
			}
			Command::SetRate(rate) => {
				self.rate = rate;
				self.with_transport(|t| t.set_rate(rate));
			}
			Command::SetTranspose(semitones) => {
				self.transpose = semitones;
				self.with_transport(|t| t.set_transpose(semitones));
			}
			Command::Shutdown => {}
		}
	}

	/// Reports the current position right away, outside the regular interval.
	fn report_position(&mut self) {
		if let Some(transport) = &self.transport {
			self.last_position_report = Instant::now();
			self.emit(PlayerEvent::Position(transport.position_us()));
		}
	}

	fn with_transport(&mut self, action: impl FnOnce(&mut Transport<ServiceClock, Connection>)) {
		match self.transport.as_mut() {
			Some(transport) => action(transport),
			None => self.emit(PlayerEvent::Error("No MIDI output is selected.".to_owned())),
		}
	}

	fn select_output(&mut self, output: &OutputSelection) {
		let session = match &self.session {
			Some(session) => session,
			None => match Session::open(SESSION_NAME) {
				Ok(session) => self.session.insert(session),
				Err(e) => return self.emit(PlayerEvent::Error(e)),
			},
		};
		match session.connect(output) {
			Ok(connection) => {
				self.silence();
				let mut transport = Transport::new(ServiceClock, connection, output.group_index);
				transport.set_rate(self.rate);
				transport.set_transpose(self.transpose);
				if let Some(song) = &self.song {
					transport.load(song.clone());
				}
				self.transport = Some(transport);
			}
			Err(e) => self.emit(PlayerEvent::Error(e)),
		}
	}

	fn load(&mut self, path: &Path) {
		match smf::load(path) {
			Ok(song) => {
				self.emit(PlayerEvent::Loaded { title: song.title.clone(), duration_us: song.duration_us });
				if let Some(transport) = &mut self.transport {
					transport.load(song.clone());
				}
				self.song = Some(song);
			}
			Err(e) => self.emit(PlayerEvent::Error(format!("Could not open {}: {e}", path.display()))),
		}
	}

	/// Stops whatever is playing so nothing is left sounding when the connection goes away.
	fn silence(&mut self) {
		if let Some(transport) = &mut self.transport {
			transport.stop();
		}
	}

	fn pump(&mut self) {
		let Some(transport) = &mut self.transport else {
			self.next_wake_us = None;
			return;
		};
		let outcome = transport.pump();
		self.next_wake_us = outcome.next_wake_us;
		let state = transport.state();
		let position = transport.position_us();
		let state_changed = state != self.reported_state;
		if state_changed {
			self.reported_state = state;
			self.emit(PlayerEvent::State(state));
		}
		if outcome.song_ended {
			self.emit(PlayerEvent::SongEnded);
		}
		let due = self.last_position_report.elapsed() >= POSITION_INTERVAL;
		if state_changed || (matches!(state, State::Playing | State::Draining) && due) {
			self.last_position_report = Instant::now();
			self.emit(PlayerEvent::Position(position));
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{sync::mpsc, time::Duration};

	use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind, num::*};
	use rumpus_core::transport::State;

	use super::*;
	use crate::midi::backend::{self, Session};

	fn write_song(dir: &std::path::Path) -> PathBuf {
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
			TrackEvent { delta: u28::new(0), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) },
		];
		let smf =
			Smf { header: Header::new(Format::SingleTrack, Timing::Metrical(u15::new(480))), tracks: vec![track] };
		let path = dir.join("three notes.mid");
		smf.save(&path).unwrap();
		path
	}

	fn collect_until(rx: &mpsc::Receiver<PlayerEvent>, stop: impl Fn(&PlayerEvent) -> bool) -> Vec<PlayerEvent> {
		let mut events = Vec::new();
		loop {
			let event = rx.recv_timeout(Duration::from_secs(5)).expect("player event");
			let done = stop(&event);
			events.push(event);
			if done {
				return events;
			}
		}
	}

	#[test]
	#[ignore = "needs the Windows MIDI Service"]
	fn plays_a_file_through_the_loopback_and_reports_progress() {
		backend::init().unwrap();
		let dir = tempfile::tempdir().unwrap();
		let path = write_song(dir.path());
		let (event_tx, events) = mpsc::channel();
		let player = spawn(move |event| {
			let _ = event_tx.send(event);
		});

		let session = Session::open("Rumpus player test").unwrap();
		let receiver =
			session.connect(&OutputSelection { endpoint_id: backend::loopback_b_id(), group_index: 0 }).unwrap();
		let (word_tx, words) = mpsc::channel();
		let _revoker = receiver.on_message(move |at, word| {
			let _ = word_tx.send((at, word));
		});

		player.send(Command::Probe);
		let outputs = collect_until(&events, |e| matches!(e, PlayerEvent::Outputs(_)));
		assert!(matches!(outputs.last(), Some(PlayerEvent::Outputs(list)) if !list.is_empty()));

		player.send(Command::SelectOutput(OutputSelection { endpoint_id: backend::loopback_a_id(), group_index: 0 }));
		player.send(Command::Load(path));
		player.send(Command::Play);
		let played = collect_until(&events, |e| matches!(e, PlayerEvent::SongEnded));
		assert!(played.iter().any(|e| matches!(e, PlayerEvent::Loaded { title, .. } if title == "three notes")));
		assert!(played.iter().any(|e| matches!(e, PlayerEvent::State(State::Playing))));
		assert!(played.iter().any(|e| matches!(e, PlayerEvent::Position(_))));
		player.shutdown();

		let mut received = Vec::new();
		while let Ok(item) = words.recv_timeout(Duration::from_millis(200)) {
			received.push(item);
		}
		// Only channel 1 note-ons: other tests share the loopback and use other channels.
		let note_ons: Vec<u32> = received.iter().map(|&(_, w)| w).filter(|w| w & 0x00FF_0000 == 0x0090_0000).collect();
		assert_eq!(note_ons, [0x2090_3C64, 0x2090_3E64, 0x2090_4064]);
		assert!(received.windows(2).all(|pair| pair[0].0 <= pair[1].0), "timestamps in order: {received:?}");
	}
}
