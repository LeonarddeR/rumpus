//! The main window: playlist, transport controls and the glue to the playback thread.

use std::{
	cell::RefCell,
	path::PathBuf,
	sync::atomic::{AtomicUsize, Ordering},
};

use rumpus_core::{
	config::{AppConfig, OutputSelection},
	playlist::Playlist,
	time_format::position_text,
	transport::{MAX_RATE, MAX_TRANSPOSE, MIN_RATE, State},
};
use wxdragon::prelude::*;

use crate::{
	announce::{accessible_name, announce, create_live_region},
	dialogs,
	ipc::{self, IpcCommand},
	menu::{self, ids},
	midi::backend::{self, OutputDevice},
	player::{self, Command, Player, PlayerEvent},
	win32::Windows::Win32::{HWND, SetForegroundWindow},
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
	/// Whether an output is selected, so files can be loaded and played.
	ready: bool,
	/// Whether the service has been probed, so outputs can be listed on the UI thread.
	probed: bool,
	/// Files opened before the service was ready, played once it is.
	pending_files: Vec<PathBuf>,
	/// Announce the next position report, because the user just sought.
	announce_next_position: bool,
}

pub struct App {
	pub frame: Frame,
	player: RefCell<Option<Player>>,
	state: RefCell<UiState>,
	menu_bar: MenuBar,
	device_menu: Menu,
	playlist_box: ListBox,
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
		let device_menu = usize::try_from(menu_bar.find_menu("Device"))
			.ok()
			.and_then(|pos| menu_bar.get_menu(pos))
			.expect("the menu bar has a Device menu");
		frame.create_status_bar(1, 0, -1, "statusbar");
		frame.set_status_text("Starting Windows MIDI Services...", 0);

		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();

		let playlist_box = labelled(panel, sizer, "&Playlist:", |p| ListBox::builder(p).build());
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
			device_menu,
			playlist_box,
			position_label,
			tempo_spin,
			transpose_spin,
			live_region,
		};
		app.set_ready(false);

		frame.on_menu_selected(|event| with_app(|app| app.on_command(event.get_id())));
		app.device_menu.bind_internal(EventType::MENU_OPEN, |event| {
			with_app(Self::reload_outputs);
			event.skip(true);
		});
		playlist_box.on_selection_changed(|_| with_app(Self::on_playlist_selection));
		playlist_box.on_item_double_clicked(|_| with_app(Self::play_selected));
		playlist_box.bind_internal(EventType::CHAR_HOOK, |event| {
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

	/// Shows the window, starts the playback thread, which probes the service, and starts serving
	/// commands from later instances.
	pub fn start(&self, server: Option<ipc::Server>) {
		self.frame.show(true);
		self.playlist_box.set_focus();
		let player = player::spawn(|event| {
			call_after(Box::new(move || with_app(|app| app.on_player_event(event))));
			wake_up_idle();
		});
		player.send(Command::Probe);
		*self.player.borrow_mut() = Some(player);
		if let Some(server) = server {
			server.serve(|command| {
				call_after(Box::new(move || with_app(|app| app.on_ipc_command(command))));
				wake_up_idle();
			});
		}
	}

	/// Brings the window to the front and queues the files a later instance was started with.
	fn on_ipc_command(&self, command: IpcCommand) {
		tracing::info!(?command, "ipc");
		self.activate();
		if let IpcCommand::OpenFiles(files) = command {
			self.open_files(files, false);
		}
	}

	fn activate(&self) {
		self.frame.show(true);
		self.frame.iconize(false);
		self.frame.request_user_attention(UserAttentionFlag::Info);
		self.frame.raise();
		let handle = self.frame.get_handle();
		if !handle.is_null() {
			// SAFETY: the handle belongs to a live frame owned by this leaked `App`.
			let _ = unsafe { SetForegroundWindow(HWND(handle)) };
		}
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
		self.state.borrow_mut().ready = ready;
		self.tempo_spin.enable(ready);
		self.transpose_spin.enable(ready);
		for id in ids::PLAYBACK {
			self.menu_bar.enable_item(id, ready);
		}
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

	/// Handles the outputs found by the probe: selects the persisted one, or the first.
	fn on_outputs(&self, outputs: Vec<OutputDevice>) {
		self.state.borrow_mut().probed = true;
		let in_use = self.show_outputs(outputs);
		if self.state.borrow().outputs.is_empty() {
			self.frame.set_status_text("No MIDI output devices found", 0);
			self.announce("No MIDI output devices found.");
			return;
		}
		let selected = in_use.unwrap_or(0);
		self.device_menu.check_item(ids::device_id(selected), true);
		self.select_output(selected);
		self.set_ready(true);
		self.frame.set_status_text("Stopped", 0);
		let pending = std::mem::take(&mut self.state.borrow_mut().pending_files);
		self.open_files(pending, true);
	}

	/// Lists the outputs again as the Device menu opens, leaving the selection alone.
	fn reload_outputs(&self) {
		if !self.state.borrow().probed {
			return;
		}
		self.show_outputs(backend::outputs());
	}

	/// Stores `outputs` and rebuilds the Device menu from them, checking the one in use.
	fn show_outputs(&self, outputs: Vec<OutputDevice>) -> Option<usize> {
		for output in &outputs {
			tracing::info!(
				name = output.name,
				endpoint = output.selection.endpoint_id,
				group = output.selection.group_index,
				"output"
			);
		}
		let mut state = self.state.borrow_mut();
		let in_use = output_in_use(&outputs, state.config.output.as_ref());
		state.outputs = outputs;
		let names: Vec<String> = state.outputs.iter().map(|o| o.name.clone()).collect();
		drop(state);
		menu::populate_devices(&self.device_menu, &names, in_use);
		in_use
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
				self.frame.set_status_text(&format!("Playing: {title}"), 0);
				self.announce(&format!("Playing {title}"));
			}
			State::Paused => {
				self.frame.set_status_text(&format!("Paused: {title}"), 0);
				self.announce("Paused");
			}
			State::Stopped => {
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
				self.open_files(files, true);
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
			ids::TEMPO_DOWN => self.step_tempo(-TEMPO_STEP),
			ids::TEMPO_UP => self.step_tempo(TEMPO_STEP),
			ids::TEMPO_RESET => self.set_tempo(DEFAULT_TEMPO, true),
			ids::TRANSPOSE_DOWN => self.step_transpose(-1),
			ids::TRANSPOSE_UP => self.step_transpose(1),
			ids::TRANSPOSE_RESET => self.set_transpose(0, true),
			ids::ABOUT => dialogs::show_about(&self.frame),
			id => {
				if let Some(index) = ids::device_index(id) {
					self.choose_output(index);
				}
			}
		}
	}

	/// Switches to the output picked from the Device menu, unless it is already in use.
	fn choose_output(&self, index: usize) {
		let in_use = {
			let state = self.state.borrow();
			state.outputs.get(index).is_some_and(|o| state.config.output.as_ref() == Some(&o.selection))
		};
		if !in_use {
			self.select_output(index);
		}
	}

	/// Adds files to the playlist, once the service is ready, and plays the first of them when
	/// `play` is set or nothing is playing or paused.
	fn open_files(&self, files: Vec<PathBuf>, play: bool) {
		if files.is_empty() {
			return;
		}
		let (ready, idle) = {
			let state = self.state.borrow();
			(state.ready, matches!(state.state, None | Some(State::Stopped)))
		};
		if !ready {
			self.state.borrow_mut().pending_files.extend(files);
			return;
		}
		let first = self.state.borrow().playlist.tracks().len();
		self.state.borrow_mut().playlist.add_files(files);
		if play || idle {
			self.state.borrow_mut().playlist.select(first);
			self.refresh_playlist();
			self.play_selected();
		} else {
			self.refresh_playlist();
		}
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

	/// Moves the tempo by `delta` percent points from where it is now.
	fn step_tempo(&self, delta: i32) {
		let percent_value = self.state.borrow().tempo_percent + delta;
		self.set_tempo(percent_value, true);
	}

	/// Moves the transposition by `delta` semitones from where it is now.
	fn step_transpose(&self, delta: i32) {
		let semitones = i32::from(self.state.borrow().transpose) + delta;
		self.set_transpose(semitones, true);
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

/// The index of `selection` among `outputs`, when it is still listed.
fn output_in_use(outputs: &[OutputDevice], selection: Option<&OutputSelection>) -> Option<usize> {
	let wanted = selection?;
	outputs.iter().position(|o| &o.selection == wanted)
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

	fn output(endpoint_id: &str, group_index: u8) -> OutputDevice {
		OutputDevice {
			name: endpoint_id.to_owned(),
			selection: OutputSelection { endpoint_id: endpoint_id.to_owned(), group_index },
		}
	}

	#[test]
	fn output_in_use_is_found_by_selection() {
		let outputs = [output("a", 0), output("b", 0), output("b", 1)];
		assert_eq!(output_in_use(&outputs, Some(&outputs[2].selection)), Some(2));
	}

	#[test]
	fn output_in_use_is_none_when_unplugged_or_unset() {
		let outputs = [output("a", 0)];
		assert_eq!(output_in_use(&outputs, Some(&output("b", 0).selection)), None);
		assert_eq!(output_in_use(&outputs, None), None);
	}
}
