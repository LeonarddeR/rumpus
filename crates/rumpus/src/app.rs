//! The main window: playlist, transport controls and the glue to the playback thread.

use std::{
	cell::RefCell,
	path::PathBuf,
	sync::atomic::{AtomicUsize, Ordering},
};

use rumpus_core::{
	config::AppConfig,
	playlist::Playlist,
	time_format::position_text,
	transport::{MAX_RATE, MAX_TRANSPOSE, MIN_RATE, State},
};
use wxdragon::prelude::*;

use crate::{
	announce::{accessible_name, announce, create_live_region},
	dialogs,
	menu::{self, ids},
	midi::backend::OutputDevice,
	player::{self, Command, Player, PlayerEvent},
};

const SEEK_STEP_US: i64 = 5_000_000;
const SEEK_FAR_STEP_US: i64 = 30_000_000;
const TEMPO_STEP: i32 = 5;
const DEFAULT_TEMPO: i32 = 100;

/// Set once from the leaked `App`; read on the main thread by callbacks that cannot capture it.
static APP_PTR: AtomicUsize = AtomicUsize::new(0);

pub fn store_app(app: &'static App) {
	APP_PTR.store(std::ptr::from_ref(app) as usize, Ordering::SeqCst);
}

pub fn app_from_ptr() -> Option<&'static App> {
	let ptr = APP_PTR.load(Ordering::SeqCst);
	if ptr == 0 {
		return None;
	}
	// SAFETY: the pointer came from a leaked `Box<App>` that lives for the rest of the process.
	unsafe { (ptr as *const App).as_ref() }
}

/// Everything the window shows, mutated only on the UI thread.
#[derive(Default)]
struct UiState {
	playlist: Playlist,
	config: AppConfig,
	outputs: Vec<OutputDevice>,
	state: Option<State>,
	title: String,
	position_us: u64,
	duration_us: u64,
	tempo_percent: i32,
	transpose: i8,
	/// Files from the command line, played once the service is ready.
	pending_files: Vec<PathBuf>,
	/// Announce the next position report, because the user just sought.
	announce_next_position: bool,
}

pub struct App {
	pub frame: Frame,
	player: RefCell<Option<Player>>,
	state: RefCell<UiState>,
	menu_bar: MenuBar,
	output_choice: Choice,
	playlist_box: ListBox,
	play_button: Button,
	position_label: TextCtrl,
	tempo_spin: SpinCtrl,
	transpose_spin: SpinCtrl,
	live_region: StaticText,
}

impl App {
	pub fn new(config: AppConfig, config_warning: Option<String>, files: Vec<PathBuf>) -> Self {
		let frame = Frame::builder().with_title("Rumpus").with_size(Size::new(700, 520)).build();
		frame.set_menu_bar(menu::create_menu_bar());
		let menu_bar = frame.get_menu_bar().expect("the menu bar was just set");
		frame.create_status_bar(1, 0, -1, "statusbar");
		frame.set_status_text("Starting Windows MIDI Services...", 0);

		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();

		let output_choice = labelled(panel, sizer, "&Output device:", |p| Choice::builder(p).build());
		let playlist_box = labelled(panel, sizer, "&Playlist:", |p| ListBox::builder(p).build());

		let buttons = BoxSizer::builder(Orientation::Horizontal).build();
		let previous_button = Button::builder(&panel).with_label("Pre&vious").build();
		let play_button = Button::builder(&panel).with_label("&Play").build();
		let stop_button = Button::builder(&panel).with_label("&Stop").build();
		let next_button = Button::builder(&panel).with_label("&Next").build();
		for button in [previous_button, play_button, stop_button, next_button] {
			buttons.add(&button, 0, SizerFlag::All, 4);
		}
		sizer.add_sizer(&buttons, 0, SizerFlag::Left | SizerFlag::All, 4);

		let position_label = labelled(panel, sizer, "Position:", |p| {
			TextCtrl::builder(p).with_value("0:00 / 0:00").with_style(TextCtrlStyle::ReadOnly).build()
		});
		let tempo_spin = labelled(panel, sizer, "&Tempo (%):", |p| {
			SpinCtrl::builder(p)
				.with_range(percent(MIN_RATE), percent(MAX_RATE))
				.with_initial_value(DEFAULT_TEMPO)
				.build()
		});
		let transpose_spin = labelled(panel, sizer, "T&ranspose (semitones):", |p| {
			SpinCtrl::builder(p)
				.with_range(-i32::from(MAX_TRANSPOSE), i32::from(MAX_TRANSPOSE))
				.with_initial_value(0)
				.build()
		});
		let live_region = create_live_region(panel);
		panel.set_sizer(sizer, true);

		let app = Self {
			frame,
			player: RefCell::new(None),
			state: RefCell::new(UiState {
				config,
				tempo_percent: DEFAULT_TEMPO,
				pending_files: files,
				..UiState::default()
			}),
			menu_bar,
			output_choice,
			playlist_box,
			play_button,
			position_label,
			tempo_spin,
			transpose_spin,
			live_region,
		};
		app.set_ready(false);

		frame.on_menu_selected(|event| with_app(|app| app.on_command(event.get_id())));
		previous_button.on_click(|_| with_app(|app| app.on_command(ids::PREVIOUS)));
		play_button.on_click(|_| with_app(|app| app.on_command(ids::PLAY_PAUSE)));
		stop_button.on_click(|_| with_app(|app| app.on_command(ids::STOP)));
		next_button.on_click(|_| with_app(|app| app.on_command(ids::NEXT)));
		output_choice.on_selection_changed(|_| with_app(Self::on_output_chosen));
		playlist_box.on_selection_changed(|_| with_app(Self::on_playlist_selection));
		playlist_box.on_item_double_clicked(|_| with_app(Self::play_selected));
		playlist_box.bind_internal(EventType::KEY_DOWN, |event| {
			let key = event.get_key_code().unwrap_or(0);
			if key == WXK_RETURN || key == WXK_NUMPAD_ENTER {
				event.skip(false);
				with_app(Self::play_selected);
			} else {
				event.skip(true);
			}
		});
		tempo_spin.on_value_changed(|_| with_app(|app| app.set_tempo(app.tempo_spin.value(), false)));
		transpose_spin.on_value_changed(|_| with_app(|app| app.set_transpose(app.transpose_spin.value(), false)));
		frame.on_close(|_| with_app(Self::on_close));

		if let Some(warning) = config_warning {
			tracing::warn!("config: {warning}");
		}
		app
	}

	/// Shows the window and starts the playback thread, which probes the service.
	pub fn start(&self) {
		self.frame.show(true);
		self.playlist_box.set_focus();
		let player = player::spawn(|event| {
			call_after(Box::new(move || with_app(|app| app.on_player_event(event))));
			wake_up_idle();
		});
		player.send(Command::Probe);
		*self.player.borrow_mut() = Some(player);
	}

	fn send(&self, command: Command) {
		if let Some(player) = self.player.borrow().as_ref() {
			player.send(command);
		}
	}

	fn announce(&self, message: &str) {
		announce(self.live_region, message);
	}

	fn set_ready(&self, ready: bool) {
		self.output_choice.enable(ready);
		self.play_button.enable(ready);
		self.tempo_spin.enable(ready);
		self.transpose_spin.enable(ready);
		for id in ids::PLAYBACK {
			self.menu_bar.enable_item(id, ready);
		}
		self.menu_bar.enable_item(ids::REFRESH_DEVICES, ready);
	}

	fn on_player_event(&self, event: PlayerEvent) {
		match event {
			PlayerEvent::Unavailable(message) => {
				tracing::error!("unavailable: {message}");
				dialogs::show_error(&self.frame, &message);
				self.frame.close(true);
			}
			PlayerEvent::Outputs(outputs) => self.on_outputs(outputs),
			PlayerEvent::Loaded { title, duration_us } => {
				let mut state = self.state.borrow_mut();
				state.title = title;
				state.duration_us = duration_us;
				state.position_us = 0;
				drop(state);
				self.refresh_position();
			}
			PlayerEvent::State(new_state) => self.on_state(new_state),
			PlayerEvent::Position(position_us) => {
				let announce_now = {
					let mut state = self.state.borrow_mut();
					state.position_us = position_us;
					std::mem::take(&mut state.announce_next_position)
				};
				self.refresh_position();
				if announce_now {
					self.announce_position();
				}
			}
			PlayerEvent::SongEnded => self.on_song_ended(),
			PlayerEvent::Error(message) => {
				tracing::error!("player: {message}");
				dialogs::show_error(&self.frame, &message);
			}
		}
	}

	fn on_outputs(&self, outputs: Vec<OutputDevice>) {
		for output in &outputs {
			tracing::info!(
				name = output.name,
				endpoint = output.selection.endpoint_id,
				group = output.selection.group_index,
				"output"
			);
		}
		let selected = {
			let mut state = self.state.borrow_mut();
			let persisted = state.config.output.clone();
			let selected = persisted.and_then(|wanted| outputs.iter().position(|o| o.selection == wanted)).unwrap_or(0);
			state.outputs = outputs;
			selected
		};
		self.output_choice.clear();
		let state = self.state.borrow();
		for output in &state.outputs {
			self.output_choice.append(&output.name);
		}
		if state.outputs.is_empty() {
			drop(state);
			self.frame.set_status_text("No MIDI output devices found", 0);
			self.announce("No MIDI output devices found.");
			return;
		}
		drop(state);
		self.output_choice.set_selection(u32::try_from(selected).unwrap_or(0));
		self.select_output(selected);
		self.set_ready(true);
		self.frame.set_status_text("Stopped", 0);
		let pending = std::mem::take(&mut self.state.borrow_mut().pending_files);
		if !pending.is_empty() {
			self.add_files(pending);
			self.play_selected();
		}
	}

	fn on_output_chosen(&self) {
		if let Some(index) = self.output_choice.get_selection() {
			self.select_output(index as usize);
		}
	}

	fn select_output(&self, index: usize) {
		let selection = {
			let mut state = self.state.borrow_mut();
			let Some(output) = state.outputs.get(index) else { return };
			let selection = output.selection.clone();
			state.config.output = Some(selection.clone());
			if let Err(e) = state.config.save() {
				tracing::warn!("saving config: {e}");
			}
			selection
		};
		self.send(Command::SelectOutput(selection));
	}

	fn on_state(&self, new_state: State) {
		tracing::info!(?new_state, "state");
		let (title, previous) = {
			let mut state = self.state.borrow_mut();
			let previous = state.state.replace(new_state);
			(state.title.clone(), previous)
		};
		match new_state {
			State::Playing => {
				self.play_button.set_label("&Pause");
				self.frame.set_status_text(&format!("Playing: {title}"), 0);
				self.announce(&format!("Playing {title}"));
			}
			State::Paused => {
				self.play_button.set_label("&Play");
				self.frame.set_status_text(&format!("Paused: {title}"), 0);
				self.announce("Paused");
			}
			State::Stopped => {
				self.play_button.set_label("&Play");
				self.frame.set_status_text("Stopped", 0);
				if previous.is_some_and(|p| p != State::Stopped) {
					self.announce("Stopped");
				}
			}
			State::Draining => {}
		}
	}

	fn on_song_ended(&self) {
		let next = self.state.borrow_mut().playlist.select_next();
		match next {
			Some(index) => {
				self.playlist_box.set_selection(u32::try_from(index).unwrap_or(0), true);
				self.play_selected();
			}
			None => self.announce("End of playlist"),
		}
	}

	fn on_command(&self, id: i32) {
		match id {
			ids::OPEN_FILES => {
				let files = dialogs::pick_midi_files(&self.frame);
				self.add_files(files);
			}
			ids::ADD_FOLDER => {
				if let Some(folder) = dialogs::pick_folder(&self.frame) {
					self.add_folder(&folder);
				}
			}
			ids::REMOVE => self.remove_selected(),
			ids::CLEAR => {
				self.send(Command::Stop);
				self.state.borrow_mut().playlist.clear();
				self.refresh_playlist();
			}
			ids::EXIT => self.frame.close(false),
			ids::PLAY_PAUSE => self.play_pause(),
			ids::STOP => self.send(Command::Stop),
			ids::PREVIOUS => self.step_track(Playlist::select_previous),
			ids::NEXT => self.step_track(Playlist::select_next),
			ids::SEEK_BACK => self.seek(-SEEK_STEP_US),
			ids::SEEK_FORWARD => self.seek(SEEK_STEP_US),
			ids::SEEK_BACK_FAR => self.seek(-SEEK_FAR_STEP_US),
			ids::SEEK_FORWARD_FAR => self.seek(SEEK_FAR_STEP_US),
			ids::ANNOUNCE_POSITION => self.announce_position(),
			ids::TEMPO_DOWN => self.set_tempo(self.state.borrow().tempo_percent - TEMPO_STEP, true),
			ids::TEMPO_UP => self.set_tempo(self.state.borrow().tempo_percent + TEMPO_STEP, true),
			ids::TEMPO_RESET => self.set_tempo(DEFAULT_TEMPO, true),
			ids::TRANSPOSE_DOWN => self.set_transpose(i32::from(self.state.borrow().transpose) - 1, true),
			ids::TRANSPOSE_UP => self.set_transpose(i32::from(self.state.borrow().transpose) + 1, true),
			ids::TRANSPOSE_RESET => self.set_transpose(0, true),
			ids::REFRESH_DEVICES => self.send(Command::RefreshOutputs),
			ids::ABOUT => dialogs::show_about(&self.frame),
			_ => {}
		}
	}

	fn add_files(&self, files: Vec<PathBuf>) {
		if files.is_empty() {
			return;
		}
		let count = files.len();
		self.state.borrow_mut().playlist.add_files(files);
		self.refresh_playlist();
		self.announce(&format!("Added {count} {}", plural(count, "file", "files")));
	}

	fn add_folder(&self, folder: &std::path::Path) {
		let added = self.state.borrow_mut().playlist.add_folder(folder);
		match added {
			Ok(count) => {
				self.refresh_playlist();
				self.announce(&format!("Added {count} {}", plural(count, "file", "files")));
			}
			Err(e) => dialogs::show_error(&self.frame, &format!("Could not read {}: {e}", folder.display())),
		}
	}

	fn remove_selected(&self) {
		let Some(index) = self.playlist_box.get_selection() else { return };
		let index = index as usize;
		let removing_current = self.state.borrow().playlist.current() == Some(index);
		if removing_current {
			self.send(Command::Stop);
		}
		self.state.borrow_mut().playlist.remove(index);
		self.refresh_playlist();
	}

	/// Rebuilds the list box from the playlist and keeps the current track selected.
	fn refresh_playlist(&self) {
		let state = self.state.borrow();
		self.playlist_box.clear();
		for track in state.playlist.tracks() {
			self.playlist_box.append(&track.title);
		}
		if let Some(current) = state.playlist.current() {
			self.playlist_box.set_selection(u32::try_from(current).unwrap_or(0), true);
		}
	}

	fn on_playlist_selection(&self) {
		if let Some(index) = self.playlist_box.get_selection() {
			self.state.borrow_mut().playlist.select(index as usize);
		}
	}

	/// Loads and plays the selected track.
	fn play_selected(&self) {
		self.on_playlist_selection();
		let path = self.state.borrow().playlist.current_track().map(|t| t.path.clone());
		if let Some(path) = path {
			self.send(Command::Load(path));
			self.send(Command::Play);
		}
	}

	fn play_pause(&self) {
		let (state, has_song) = {
			let state = self.state.borrow();
			(state.state, !state.title.is_empty())
		};
		match state {
			Some(State::Playing) => self.send(Command::Pause),
			Some(State::Paused) if has_song => self.send(Command::Play),
			_ => self.play_selected(),
		}
	}

	fn step_track(&self, step: fn(&mut Playlist) -> Option<usize>) {
		let moved = step(&mut self.state.borrow_mut().playlist);
		if let Some(index) = moved {
			self.playlist_box.set_selection(u32::try_from(index).unwrap_or(0), true);
			self.play_selected();
		}
	}

	fn seek(&self, delta_us: i64) {
		self.state.borrow_mut().announce_next_position = true;
		self.send(Command::SeekBy(delta_us));
	}

	fn refresh_position(&self) {
		let state = self.state.borrow();
		self.position_label.set_value(&position_text(state.position_us, state.duration_us));
	}

	fn announce_position(&self) {
		let state = self.state.borrow();
		self.announce(&position_text(state.position_us, state.duration_us));
	}

	fn set_tempo(&self, percent_value: i32, announce: bool) {
		let percent_value = percent_value.clamp(percent(MIN_RATE), percent(MAX_RATE));
		self.state.borrow_mut().tempo_percent = percent_value;
		if self.tempo_spin.value() != percent_value {
			self.tempo_spin.set_value(percent_value);
		}
		self.send(Command::SetRate(f64::from(percent_value) / 100.0));
		if announce {
			self.announce(&format!("Tempo {percent_value}%"));
		}
	}

	fn set_transpose(&self, semitones: i32, announce: bool) {
		let semitones = semitones.clamp(-i32::from(MAX_TRANSPOSE), i32::from(MAX_TRANSPOSE));
		let transpose = i8::try_from(semitones).unwrap_or(0);
		self.state.borrow_mut().transpose = transpose;
		if self.transpose_spin.value() != semitones {
			self.transpose_spin.set_value(semitones);
		}
		self.send(Command::SetTranspose(transpose));
		if announce {
			self.announce(&transpose_text(transpose));
		}
	}

	fn on_close(&self) {
		if let Some(player) = self.player.borrow_mut().take() {
			player.shutdown();
		}
	}
}

fn with_app(action: impl FnOnce(&'static App)) {
	if let Some(app) = app_from_ptr() {
		action(app);
	}
}

/// Adds a visible label and the control it names, mirroring the label into the accessible name.
fn labelled<W: WxWidget + Copy>(panel: Panel, sizer: BoxSizer, label: &str, build: impl FnOnce(&Panel) -> W) -> W {
	let text = StaticText::builder(&panel).with_label(label).build();
	let control = build(&panel);
	control.set_accessibility_label(&accessible_name(label));
	sizer.add(&text, 0, SizerFlag::Left | SizerFlag::Top, 6);
	sizer.add(&control, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 6);
	control
}

#[expect(clippy::cast_possible_truncation)]
const fn percent(rate: f64) -> i32 {
	(rate * 100.0) as i32
}

const fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
	if count == 1 { one } else { many }
}

fn transpose_text(semitones: i8) -> String {
	match semitones {
		0 => "Transpose off".to_owned(),
		1 => "Transpose up 1 semitone".to_owned(),
		-1 => "Transpose down 1 semitone".to_owned(),
		n if n > 0 => format!("Transpose up {n} semitones"),
		n => format!("Transpose down {} semitones", -n),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn transpose_text_reads_naturally() {
		assert_eq!(transpose_text(0), "Transpose off");
		assert_eq!(transpose_text(1), "Transpose up 1 semitone");
		assert_eq!(transpose_text(-3), "Transpose down 3 semitones");
	}

	#[test]
	fn tempo_range_is_whole_percent() {
		assert_eq!(percent(MIN_RATE), 25);
		assert_eq!(percent(MAX_RATE), 400);
	}
}
