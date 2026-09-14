//! Single-instance handling: the first instance owns a named pipe, later instances forward their
//! command line over it and exit.

use std::{env, fmt, path::PathBuf, thread};

use windows_core::{Error, HSTRING, WIN32_ERROR};

use crate::win32::Windows::Win32::{
	ASFW_ANY, AllowSetForegroundWindow, CloseHandle, ConnectNamedPipe, CreateFileW, CreateNamedPipeW,
	DisconnectNamedPipe, ERROR_ACCESS_DENIED, ERROR_PIPE_CONNECTED, FILE_ATTRIBUTE_NORMAL,
	FILE_FLAG_FIRST_PIPE_INSTANCE, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, OPEN_EXISTING, PIPE_ACCESS_INBOUND,
	PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, ReadFile, WaitNamedPipeW, WriteFile,
};

const PIPE_PREFIX: &str = r"\\.\pipe\rumpus_";
const ACTIVATE: &str = "ACTIVATE";
const BUFFER_SIZE: u32 = 64 * 1024;
const CHUNK_SIZE: u32 = 4096;
const CONNECT_TIMEOUT_MS: u32 = 2000;
const SERVER_OPEN_MODE: u32 = (PIPE_ACCESS_INBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE).unsigned_abs();
const SERVER_PIPE_MODE: u32 = (PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT).unsigned_abs();
const CLIENT_ACCESS: u32 = GENERIC_WRITE.unsigned_abs();
const CLIENT_DISPOSITION: u32 = OPEN_EXISTING.unsigned_abs();
const CLIENT_ATTRIBUTES: u32 = FILE_ATTRIBUTE_NORMAL.unsigned_abs();
const ACCESS_DENIED: WIN32_ERROR = WIN32_ERROR(ERROR_ACCESS_DENIED.unsigned_abs());
const PIPE_CONNECTED: WIN32_ERROR = WIN32_ERROR(ERROR_PIPE_CONNECTED.unsigned_abs());
const MAX_INSTANCES: u32 = PIPE_UNLIMITED_INSTANCES.unsigned_abs();

/// What a later instance asks the running one to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcCommand {
	Activate,
	OpenFiles(Vec<PathBuf>),
}

#[derive(Debug)]
pub enum ClaimError {
	AnotherInstance,
	Os(Error),
}

impl fmt::Display for ClaimError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::AnotherInstance => f.write_str("another instance owns the pipe"),
			Self::Os(e) => write!(f, "{e}"),
		}
	}
}

/// The server end of the pipe, held by the first instance for the life of the process.
pub struct Server(HANDLE);

// SAFETY: the handle is only ever used from the thread that `serve` spawns.
unsafe impl Send for Server {}

impl Server {
	const fn handle(&self) -> HANDLE {
		self.0
	}

	/// Accepts one connection at a time and hands each decoded command to `on_command` on a
	/// background thread.
	pub fn serve(self, on_command: impl Fn(IpcCommand) + Send + 'static) {
		thread::spawn(move || {
			let handle = self.handle();
			loop {
				// SAFETY: the handle is a pipe server instance owned by this thread.
				let connected =
					unsafe { ConnectNamedPipe(handle, None) }.as_bool() || WIN32_ERROR::from_thread() == PIPE_CONNECTED;
				if connected && let Some(command) = decode(&read_all(handle)) {
					on_command(command);
				}
				// SAFETY: as above.
				let _ = unsafe { DisconnectNamedPipe(handle) };
			}
		});
	}
}

/// Creates the pipe, which only the first instance in a session can do.
pub fn claim() -> Result<Server, ClaimError> {
	let name = HSTRING::from(pipe_name());
	// SAFETY: the name is a valid, nul-terminated wide string.
	let handle = unsafe {
		CreateNamedPipeW(&name, SERVER_OPEN_MODE, SERVER_PIPE_MODE, MAX_INSTANCES, BUFFER_SIZE, BUFFER_SIZE, 0, None)
	};
	if handle != INVALID_HANDLE_VALUE {
		return Ok(Server(handle));
	}
	let error = WIN32_ERROR::from_thread();
	if error == ACCESS_DENIED {
		Err(ClaimError::AnotherInstance)
	} else {
		Err(ClaimError::Os(Error::from_hresult(error.to_hresult())))
	}
}

/// Sends `files` to the running instance, or an activate request when there are none.
pub fn forward(files: &[PathBuf]) -> Result<(), Error> {
	let payload = encode(files);
	let name = HSTRING::from(pipe_name());
	// SAFETY: plain Win32 calls with valid arguments; the handle is closed before returning.
	unsafe {
		let _ = AllowSetForegroundWindow(ASFW_ANY);
		let _ = WaitNamedPipeW(&name, CONNECT_TIMEOUT_MS);
		let handle = CreateFileW(&name, CLIENT_ACCESS, 0, None, CLIENT_DISPOSITION, CLIENT_ATTRIBUTES, None);
		if handle == INVALID_HANDLE_VALUE {
			return Err(Error::from_thread());
		}
		let bytes = payload.as_bytes();
		let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
		let mut written = 0u32;
		let result = WriteFile(handle, Some(bytes.as_ptr().cast()), len, Some(&raw mut written), None).ok();
		let _ = CloseHandle(handle);
		result
	}
}

/// Reads until the client closes its end.
fn read_all(handle: HANDLE) -> Vec<u8> {
	let mut data = Vec::new();
	let mut chunk = [0u8; CHUNK_SIZE as usize];
	loop {
		let mut read = 0u32;
		// SAFETY: the buffer outlives the call and its length is passed along.
		let ok = unsafe { ReadFile(handle, Some(chunk.as_mut_ptr().cast()), CHUNK_SIZE, Some(&raw mut read), None) };
		if !ok.as_bool() || read == 0 {
			return data;
		}
		data.extend_from_slice(&chunk[..read as usize]);
	}
}

/// Named pipe path scoped to the current user; the default pipe security descriptor further
/// restricts connections to the same user.
fn pipe_name() -> String {
	let user = env::var("USERNAME").unwrap_or_else(|_| "user".to_owned());
	format!("{PIPE_PREFIX}{user}")
}

/// One path per line, or the activate marker when there are no files.
fn encode(files: &[PathBuf]) -> String {
	if files.is_empty() {
		return ACTIVATE.to_owned();
	}
	files.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join("\n")
}

fn decode(data: &[u8]) -> Option<IpcCommand> {
	let payload = String::from_utf8_lossy(data);
	let lines: Vec<&str> =
		payload.lines().map(|l| l.trim_matches(['\0', ' ', '\t', '\r'])).filter(|l| !l.is_empty()).collect();
	match lines.as_slice() {
		[] => None,
		[marker] if *marker == ACTIVATE => Some(IpcCommand::Activate),
		paths => Some(IpcCommand::OpenFiles(paths.iter().map(PathBuf::from).collect())),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decode_ignores_empty_and_nul_padding() {
		assert_eq!(decode(b""), None);
		assert_eq!(decode(b"\0\0\n"), None);
	}

	#[test]
	fn decode_recognises_activate() {
		assert_eq!(decode(b"ACTIVATE"), Some(IpcCommand::Activate));
		assert_eq!(decode(b"ACTIVATE\r\n"), Some(IpcCommand::Activate));
	}

	#[test]
	fn decode_keeps_every_path_including_spaces() {
		let files = vec![PathBuf::from(r"C:\Music\a song.mid"), PathBuf::from(r"D:\b.midi")];
		assert_eq!(decode(encode(&files).as_bytes()), Some(IpcCommand::OpenFiles(files)));
	}

	#[test]
	fn encode_marks_an_empty_list_as_activate() {
		assert_eq!(encode(&[]), ACTIVATE);
	}

	#[test]
	fn decode_treats_non_utf8_lossily() {
		let command = decode(b"C:\\\xff\\x.mid").expect("a path");
		assert!(matches!(command, IpcCommand::OpenFiles(paths) if paths.len() == 1));
	}
}
