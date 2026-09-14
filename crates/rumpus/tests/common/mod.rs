//! Shared harness for UI Automation tests: launches the real binary with its config directory,
//! single-instance pipe and MIDI outputs isolated, then drives it through the `uiautomation`
//! crate.

#![allow(dead_code, unused_imports, clippy::must_use_candidate)]

#[allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	clippy::all,
	clippy::pedantic,
	clippy::nursery
)]
mod win32;

use std::{
	path::{Path, PathBuf},
	process::{Child, Command, ExitStatus},
	time::{Duration, Instant},
};

use uiautomation::{
	UIAutomation, UITreeWalker,
	controls::ControlType,
	core::UIElement,
	inputs::Keyboard,
	patterns::{UISelectionPattern, UIValuePattern},
};
use win32::Windows::Win32::{
	INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC,
	MapVirtualKeyW, SendInput,
};
pub use win32::Windows::Win32::{VK_CONTROL, VK_DOWN, VK_RETURN, VK_SHIFT, VK_UP};

/// Accessible names of the main window's controls.
pub const PLAYLIST: &str = "Playlist";
pub const POSITION: &str = "Position";
/// The text part of the tempo spin control, which carries the value.
pub const TEMPO_EDIT: &str = "Tempo (%):";
/// The up-down part of the tempo spin control, which carries the enabled state.
pub const TEMPO_SPINNER: &str = "Tempo (%)";
pub const TRANSPOSE_EDIT: &str = "Transpose (semitones):";
pub const TRANSPOSE_SPINNER: &str = "Transpose (semitones)";
/// Names of the outputs listed while `RUMPUS_LOOPBACK_OUTPUTS` is set.
pub const LOOPBACK_A: &str = "Service Test Loopback A";
pub const LOOPBACK_B: &str = "Service Test Loopback B";

const LOG_FILE: &str = "rumpus.log";
const CONFIG_FILE: &str = "config.toml";
const POLL: Duration = Duration::from_millis(250);
pub const DEADLINE: Duration = Duration::from_secs(10);
const STARTUP_DEADLINE: Duration = Duration::from_secs(30);

pub struct App {
	pub child: Child,
	pub pid: u32,
	base: PathBuf,
}

impl App {
	/// The directory the app was told to keep its config and log in.
	pub fn config_dir(&self) -> PathBuf {
		self.base.join("config")
	}

	/// Everything the app has logged so far.
	pub fn logs(&self) -> String {
		std::fs::read_to_string(self.config_dir().join(LOG_FILE)).unwrap_or_default()
	}

	/// The saved config file, empty until the app has written it.
	pub fn config(&self) -> String {
		std::fs::read_to_string(self.config_dir().join(CONFIG_FILE)).unwrap_or_default()
	}

	pub fn is_running(&mut self) -> bool {
		self.child.try_wait().expect("try_wait").is_none()
	}

	/// Polls until the app exits; returns its exit status.
	pub fn wait_exit(&mut self, timeout: Duration) -> ExitStatus {
		let deadline = Instant::now() + timeout;
		loop {
			if let Some(status) = self.child.try_wait().expect("try_wait") {
				return status;
			}
			assert!(Instant::now() < deadline, "app did not exit within {timeout:?}");
			std::thread::sleep(POLL);
		}
	}

	/// A command line for another instance in the same isolated environment, which forwards to
	/// this one over the single-instance pipe and exits.
	pub fn another_instance(&self) -> Command {
		command(&self.base)
	}
}

impl Drop for App {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
		let _ = std::fs::remove_dir_all(&self.base);
	}
}

/// The fixture songs written for one test.
pub struct Fixtures {
	/// One and a half seconds long.
	pub three_notes: PathBuf,
	/// Ten seconds long.
	pub long_note: PathBuf,
}

/// Creates the isolated directory for `test_name` and writes the fixture songs into it; pass the
/// directory to `launch`.
pub fn prepare(test_name: &str) -> (PathBuf, Fixtures) {
	let base = std::env::temp_dir().join(format!("rumpus-ui-{test_name}-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&base);
	let songs = base.join("fixtures");
	let fixtures = Fixtures {
		three_notes: rumpus_core::testing::write_three_notes(&songs),
		long_note: rumpus_core::testing::write_long_note(&songs),
	};
	std::fs::create_dir_all(base.join("config")).expect("create the config dir");
	(base, fixtures)
}

/// Launches the app with `files` on its command line, isolated in `base` from `prepare`.
pub fn launch(base: PathBuf, files: &[&Path]) -> App {
	let child = command(&base).args(files).spawn().expect("launch rumpus");
	let pid = child.id();
	App { child, pid, base }
}

/// Launches the app with nothing on its command line.
pub fn launch_empty(test_name: &str) -> App {
	let (base, _) = prepare(test_name);
	launch(base, &[])
}

/// The app binary with its config and log in `base/config`, only the diagnostic loopback
/// outputs listed, debug logging, and the single-instance pipe keyed on the base directory's
/// name instead of the real user name.
fn command(base: &Path) -> Command {
	let instance = base.file_name().expect("the base dir has a name").to_os_string();
	let mut command = Command::new(env!("CARGO_BIN_EXE_rumpus"));
	command
		.env("RUMPUS_CONFIG_DIR", base.join("config"))
		.env("RUMPUS_LOOPBACK_OUTPUTS", "1")
		.env("RUST_LOG", "rumpus=debug")
		.env("USERNAME", instance);
	command
}

fn automation() -> UIAutomation {
	UIAutomation::new().expect("UI Automation unavailable")
}

fn millis(duration: Duration) -> u64 {
	duration.as_millis().try_into().expect("timeout fits")
}

/// Polls `observe` until `condition` holds for its result; panics with `what` and the last
/// observation on timeout.
pub fn wait_until<T: std::fmt::Debug>(
	what: &str,
	timeout: Duration,
	mut observe: impl FnMut() -> T,
	condition: impl Fn(&T) -> bool,
) -> T {
	let deadline = Instant::now() + timeout;
	loop {
		let observed = observe();
		if condition(&observed) {
			return observed;
		}
		assert!(Instant::now() < deadline, "{what} not reached within {timeout:?}; last seen {observed:?}");
		std::thread::sleep(POLL);
	}
}

/// The focused element anywhere on the desktop: its process id, type and name.
fn focused_anywhere() -> Option<(u32, ControlType, String)> {
	let el = automation().get_focused_element().ok()?;
	Some((el.get_process_id().ok()?, el.get_control_type().ok()?, el.get_name().ok()?))
}

/// The app's focused element type and name, or None while another process has focus or the
/// window is not up yet.
pub fn focused(pid: u32) -> Option<(ControlType, String)> {
	focused_anywhere().filter(|(owner, ..)| *owner == pid).map(|(_, control_type, name)| (control_type, name))
}

/// Polls the focused element until `expected` matches it; panics on timeout, naming whatever
/// had the focus instead.
pub fn wait_for_focus(
	pid: u32,
	timeout: Duration,
	expected: impl Fn(ControlType, &str) -> bool,
) -> (ControlType, String) {
	let (_, control_type, name) = wait_until("expected focus", timeout, focused_anywhere, |last| {
		last.as_ref().is_some_and(|(owner, control_type, name)| *owner == pid && expected(*control_type, name))
	})
	.expect("focus observed");
	(control_type, name)
}

pub fn is_playlist(control_type: ControlType, name: &str) -> bool {
	control_type == ControlType::List && name == PLAYLIST
}

/// Waits until the window is up with the playlist focused and the service has been probed, which
/// enables the tempo control, activating the window first when the focus is elsewhere.
pub fn wait_ready(app: &App) {
	let window = main_window(app.pid);
	if focused(app.pid).is_none() {
		window.set_focus().expect("activate the app window");
	}
	wait_for_focus(app.pid, STARTUP_DEADLINE, |control_type, name| {
		is_playlist(control_type, name) || control_type == ControlType::ListItem
	});
	let tempo = find_widget(app.pid, ControlType::Spinner, TEMPO_SPINNER);
	wait_for_enabled(&tempo, true);
}

/// The app's main window.
pub fn main_window(pid: u32) -> UIElement {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	automation
		.create_matcher()
		.from(root)
		.depth(2)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Window)
		.name("Rumpus")
		.timeout(millis(STARTUP_DEADLINE))
		.find_first()
		.expect("app window")
}

/// Finds a widget by control type and name inside the main window.
pub fn find_widget(pid: u32, control_type: ControlType, name: &str) -> UIElement {
	find_widget_in(&main_window(pid), control_type, name)
}

/// Finds a widget by control type and name inside `container`, such as a dialog.
pub fn find_widget_in(container: &UIElement, control_type: ControlType, name: &str) -> UIElement {
	automation()
		.create_matcher()
		.from(container.clone())
		.depth(5)
		.control_type(control_type)
		.name(name)
		.timeout(millis(DEADLINE))
		.find_first()
		.unwrap_or_else(|e| panic!("widget {control_type:?} {name:?} not found: {e}"))
}

/// Finds a top-level window of `pid` titled `title`. Depth 3, not 2: a modal dialog nests under
/// its owner window in the UIA tree, one level deeper than the frame itself.
pub fn find_window(pid: u32, title: &str) -> UIElement {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	automation
		.create_matcher()
		.from(root)
		.depth(3)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Window)
		.name(title)
		.timeout(millis(DEADLINE))
		.find_first()
		.unwrap_or_else(|e| panic!("window {title:?} not found: {e}"))
}

/// Waits for the popup of the menu-bar menu `name` to be open.
pub fn wait_for_popup(pid: u32, name: &str) -> UIElement {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	automation
		.create_matcher()
		.from(root)
		.depth(3)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Menu)
		.name(name)
		.timeout(millis(DEADLINE))
		.find_first()
		.unwrap_or_else(|e| panic!("popup menu {name:?} not found: {e}"))
}

pub fn value(element: &UIElement) -> String {
	let pattern: UIValuePattern = element.get_pattern().expect("ValuePattern");
	pattern.get_value().expect("get value")
}

/// Polls the element's value until it reads `expected`; panics on timeout.
pub fn wait_for_value(element: &UIElement, expected: &str) {
	wait_for_value_where(element, |current| current == expected);
}

/// Polls the element's value until `accept` holds for it; returns that value.
pub fn wait_for_value_where(element: &UIElement, accept: impl Fn(&str) -> bool) -> String {
	wait_until("expected value", DEADLINE, || value(element), |current| accept(current))
}

/// Polls the element's enabled state until it equals `expected`.
pub fn wait_for_enabled(element: &UIElement, expected: bool) {
	wait_until(
		"enabled state",
		STARTUP_DEADLINE,
		|| element.is_enabled().expect("is_enabled"),
		|&enabled| enabled == expected,
	);
}

fn selected_names(list: &UIElement) -> Vec<String> {
	let selection: UISelectionPattern = list.get_pattern().expect("SelectionPattern");
	selection.get_selection().expect("selection").iter().map(|item| item.get_name().expect("item name")).collect()
}

/// Polls the list's selected item names until they equal `expected`; panics on timeout.
pub fn wait_for_selection(list: &UIElement, expected: &[&str]) {
	wait_until("expected selection", DEADLINE, || selected_names(list), |names| names == expected);
}

/// The matcher counts the root as depth 1, so the rows directly below the list sit at depth 2.
fn item_count(list: &UIElement) -> usize {
	automation()
		.create_matcher()
		.from(list.clone())
		.depth(2)
		.control_type(ControlType::ListItem)
		.timeout(1_000)
		.find_all()
		.map_or(0, |items| items.len())
}

/// Polls the list's row count until it equals `expected`; panics on timeout.
pub fn wait_for_item_count(list: &UIElement, expected: usize) {
	wait_until("expected row count", DEADLINE, || item_count(list), |&count| count == expected);
}

/// The status bar's text: the name of its first field, walked in the control view.
pub fn status_text(pid: u32) -> String {
	let bar = find_widget_in(&main_window(pid), ControlType::StatusBar, "");
	let walker = automation().get_control_view_walker().expect("walker");
	walker.get_first_child(&bar).and_then(|field| field.get_name()).expect("status bar text")
}

/// Polls the status bar until it reads `expected`; panics on timeout.
pub fn wait_for_status(pid: u32, expected: &str) {
	wait_until("expected status", DEADLINE, || status_text(pid), |current| current == expected);
}

/// Polls the app's log until a line contains `needle`; returns that line.
pub fn wait_for_log(app: &App, needle: &str) -> String {
	wait_for_log_where(app, |line| line.contains(needle))
}

/// Polls the app's log until a line satisfies `accept`; returns that line.
pub fn wait_for_log_where(app: &App, accept: impl Fn(&str) -> bool) -> String {
	wait_until(
		"expected log line",
		DEADLINE,
		|| app.logs().lines().find(|line| accept(line)).map(str::to_owned),
		Option::is_some,
	)
	.expect("log line observed")
}

/// Polls the saved config until `accept` holds for it; returns that config.
pub fn wait_for_config_where(app: &App, accept: impl Fn(&str) -> bool) -> String {
	wait_until("expected config", DEADLINE, || app.config(), |current| accept(current))
}

/// Sends a key chord OS-wide, after checking that the app still owns the focus.
pub fn send_keys(pid: u32, keys: &str) {
	assert!(focused(pid).is_some(), "the app lost the foreground before sending {keys:?}");
	Keyboard::new().send_keys(keys).expect("send keys");
}

/// Presses and releases `key` with `modifiers` held, as scan-coded key events, after the
/// same foreground check as `send_keys`.
pub fn press(pid: u32, modifiers: &[i32], key: i32) {
	assert!(focused(pid).is_some(), "the app lost the foreground before pressing {key:#x}");
	let mut inputs: Vec<INPUT> = modifiers.iter().map(|&m| key_input(m, 0)).collect();
	inputs.push(key_input(key, 0));
	inputs.push(key_input(key, KEYEVENTF_KEYUP));
	inputs.extend(modifiers.iter().rev().map(|&m| key_input(m, KEYEVENTF_KEYUP)));
	// SAFETY: the slice outlives the call and the size is that of its element type.
	let sent = unsafe { SendInput(&inputs, i32::try_from(std::mem::size_of::<INPUT>()).expect("INPUT size")) };
	assert_eq!(sent as usize, inputs.len(), "SendInput rejected the key events");
}

/// The navigation cluster keys, which carry the extended flag on a real keyboard.
const fn is_extended(key: i32) -> bool {
	matches!(key, VK_UP | VK_DOWN)
}

fn key_input(key: i32, flags: i32) -> INPUT {
	let vk = u16::try_from(key).expect("virtual key");
	// SAFETY: plain table lookup with a valid virtual key.
	let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC as u32) };
	let flags = if is_extended(key) { flags | KEYEVENTF_EXTENDEDKEY } else { flags };
	INPUT {
		r#type: INPUT_KEYBOARD.cast_unsigned(),
		Anonymous: INPUT_0 {
			ki: KEYBDINPUT {
				wVk: vk,
				wScan: u16::try_from(scan).expect("scan code"),
				dwFlags: flags.cast_unsigned(),
				time: 0,
				dwExtraInfo: 0,
			},
		},
	}
}

/// Prints every element below `root`, one per line, for exploring the tree while writing tests.
pub fn dump(root: &UIElement) {
	fn visit(walker: &UITreeWalker, element: &UIElement, depth: usize) {
		let indent = "  ".repeat(depth);
		let control_type = element.get_control_type().map(|t| format!("{t:?}")).unwrap_or_default();
		let name = element.get_name().unwrap_or_default();
		let class = element.get_classname().unwrap_or_default();
		let enabled = element.is_enabled().unwrap_or(false);
		let value = element
			.get_pattern::<UIValuePattern>()
			.ok()
			.and_then(|p| p.get_value().ok())
			.map(|v| format!(" value={v:?}"))
			.unwrap_or_default();
		println!("{indent}{control_type} {name:?} class={class:?} enabled={enabled}{value}");
		if let Ok(child) = walker.get_first_child(element) {
			visit(walker, &child, depth + 1);
			let mut next = child;
			while let Ok(sibling) = walker.get_next_sibling(&next) {
				visit(walker, &sibling, depth + 1);
				next = sibling;
			}
		}
	}
	let walker = automation().get_control_view_walker().expect("walker");
	visit(&walker, root, 0);
}
