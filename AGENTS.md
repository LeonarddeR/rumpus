# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project

Rumpus is an accessible MIDI file player for Windows 11, built for keyboard and screen reader users. It plays Standard MIDI Files through **Windows MIDI Services** (the `Windows.Devices.Midi2` WinRT API), never through the legacy WinMM API. The GUI is wxWidgets via `wxdragon`. MSRV is `1.98`, edition 2024.

## Build, lint, test

All `cargo` commands that touch the `rumpus` crate must run inside a **Visual Studio Developer Command Prompt**: `wxdragon-sys` downloads wxWidgets and builds it from source with CMake and Ninja on the first build. `rumpus-core` and `xtask` build anywhere, including Linux (CI lints and tests them on `ubuntu-latest`).

```powershell
.\tools\fetch-sdk.ps1                       # once: fetches the Windows MIDI Services SDK into sdk/
cargo run -p rumpus -- path\to\song.mid
cargo fmt --all -- --check
cargo clippy -p rumpus-core -p xtask --all-targets -- -D warnings
cargo clippy -p rumpus --all-targets -- -D warnings
cargo test -p rumpus-core                   # pure logic, no Windows needed
cargo test -p rumpus                        # unit tests that need no MIDI service
cargo test -p rumpus -- --ignored           # needs the MIDI service and an interactive desktop
cargo test -p rumpus-core transport::       # one module
cargo test -p rumpus --test startup -- --ignored   # one UI test file
cargo bindgen                               # regenerate all committed bindings
cargo xtask build-windows                   # release exe for x86_64-pc-windows-msvc
cargo xtask dist                            # stage files and build the Inno Setup installer
```

`build.rs` of the `rumpus` crate fails with a pointer to `tools\fetch-sdk.ps1` when `sdk/runtimes/` is missing. It copies `Windows.Devices.Midi2.dll` and `.pri` next to the executable **and** into `deps/`, because test executables run from there and WinRT activation finds the classes through the DLL beside the binary when they are not registered system-wide.

Style: hard tabs, width 120 (`rustfmt.toml`). Workspace lints deny `clippy::all` and warn on `pedantic` and `nursery`; CI passes `-D warnings`, so every warning is an error. Silence a lint with `#[expect(...)]` on the narrowest item, as the existing code does.

## Architecture

### Two crates, one boundary

- `crates/rumpus-core` holds all playback logic and has **no dependency on Windows or any UI toolkit**: SMF parsing (`smf`, on `midly`), the `Song` model, UMP word building (`ump`, on `midi2`), the `Transport` state machine, `Playlist`, `AppConfig` and time formatting. Anything that can be decided without the OS belongs here, where it is unit tested against fake clocks and sinks.
- `crates/rumpus` is the Windows application: the wx window, the playback thread, the WinRT backend and the single-instance pipe.

The seam between them is two traits in `transport.rs`: `Clock` (monotonic microseconds) and `Sink` (deliver UMP words at an absolute clock time). `midi::backend` implements them as `ServiceClock` and `Connection` on top of the MIDI service; tests implement them with fakes.

### Transport (`rumpus-core/src/transport.rs`)

`Transport` does not play in real time. Each `pump()` hands the sink every event that falls within `WINDOW_US` (500 ms) ahead of the clock, stamped with the absolute time it must sound; the MIDI service does the precise scheduling. `pump()` returns `next_wake_us`, which tells the caller when to pump again.

Points that need more than one file to see:

- Two anchors map song time to wall time. `position_anchor` drives the position readout, `feed_anchor` drives scheduling. They diverge after a rate change, because events already handed to the service keep their timestamps.
- Because events are scheduled ahead, stopping, pausing, seeking and loading cannot take effect instantly. They schedule silence after the last sent event and enter `State::Draining` until that silence has landed, then move to the pending state (`after_drain`).
- After a seek, per-channel state (bank select, program, controllers, pitch bend, pressure) is replayed so the synth sounds as if it had played through. Bank select goes before the program change.
- Transpose skips the drum channel, and sounding notes are tracked by the key as written so their note-offs match the key actually sent.

### Threads in the application

- **UI thread** runs wx. `App` is created inside `wxdragon::main`, leaked with `Box::leak` and stored in a static pointer (`store_app` / `app_from_ptr`), because wx event closures cannot capture it. All window state lives in `RefCell<UiState>` and is touched only on this thread; handlers reach it through `with_app`.
- **Player thread** (`player.rs`, named `rumpus-player`) owns the WinRT `Session` and the `Transport`. It joins the MTA on start (`backend::init`), then loops on `recv_timeout(next_wake)` followed by `pump()`. The UI sends `Command`s over an `mpsc` channel; the thread reports `PlayerEvent`s through a callback.
- **IPC thread** serves the single-instance pipe.

Both background threads get back to the UI thread the same way: `call_after(...)` followed by `wake_up_idle()`. Never touch a wx object from them directly.

On shutdown and whenever the output changes, `Worker::silence` stops the transport, pumps until the drain finishes, then sleeps `DISCONNECT_GRACE` so the service delivers the all-notes-off before the connection closes. Dropping the connection early leaves notes hanging.

### Windows MIDI Services backend (`midi/backend.rs`)

`probe()` distinguishes three failures: API not installed, PC in legacy API mode, service not running. The same check backs the `rumpus.exe --probe` flag, whose exit code (0 usable, 1 legacy mode, 2 service unavailable, 3 not installed) the Inno Setup script reads before its first page. Keep `probe_exit_code` and `dist/windows/installer.iss` in step.

An output is not an endpoint but a **MIDI 1.0 port on an endpoint**: one entry of the endpoint's port name table with destination flow, identified by `OutputSelection { endpoint_id, group_index }`. An endpoint without a name table is listed once on group 0. That selection is the only thing persisted in `config.toml`. The Device menu is rebuilt from a fresh enumeration every time it opens.

### Single instance (`ipc.rs`)

The first instance creates a named pipe `\\.\pipe\rumpus_<USERNAME>` with `FILE_FLAG_FIRST_PIPE_INSTANCE`. A later instance fails to claim it, forwards its absolute file paths (or `ACTIVATE`) over the pipe and exits. The running instance raises its window and adds the files to the playlist. A failure to claim or forward is logged and the app starts anyway.

### Accessibility

Playback must never interrupt a screen reader. The position is a read-only text field that is announced only on request (Ctrl+I) or right after a seek. Everything else that needs speaking goes through `announce.rs`, a hidden `StaticText` registered as a UIA live region with the `live-region` crate. Announcements are also logged at `debug` level, which is how the UI tests assert on them. Every control gets a visible label with a mnemonic, and its accessible name is derived from that label by `accessible_name`.

### Logging and environment variables

The process uses the `windows` subsystem and has no console, so logs go to `rumpus.log` in the config directory. `RUST_LOG` sets the level (default `info`).

- `RUMPUS_CONFIG_DIR` overrides the config and log directory.
- `RUMPUS_LOOPBACK_OUTPUTS` restricts the output list to the service's diagnostic loopback endpoints.

## Tests

- `rumpus-core` tests are plain unit tests. `rumpus_core::testing` writes fixture MIDI files and is a public module so the GUI crate's tests can use it too.
- Tests marked `#[ignore]` need the Windows MIDI Service. They use its built-in diagnostic loopback endpoints (send on loopback A, receive on loopback B), so no MIDI hardware is required. Tests share that loopback, so each asserts only on its own channel.
- `crates/rumpus/tests/*.rs` are UI tests that launch the real `rumpus.exe` and drive it through UI Automation. They need an interactive desktop and take over the keyboard. **One test per file**, because each test launches the app and owns the foreground. The harness in `tests/common/` isolates each run with `RUMPUS_CONFIG_DIR`, `RUMPUS_LOOPBACK_OUTPUTS`, `RUST_LOG` and a unique `USERNAME` (which moves the single-instance pipe away from any copy of Rumpus the developer has open). Key presses are sent with scan codes through `SendInput`.
- CI never runs the ignored tests. They run locally and in the `cargo release` pre-release hook.

## Generated bindings

Three files are generated by `windows-bindgen` and committed. Never edit them by hand:

| File | Filter list | Source metadata |
| --- | --- | --- |
| `crates/rumpus/src/midi/bindings.rs` | `tools/bindgen/midi.txt` | `sdk/Windows.Devices.Midi2.winmd` |
| `crates/rumpus/src/win32.rs` | `tools/bindgen/win32.txt` | metadata shipped with `windows-bindgen` |
| `crates/rumpus/tests/common/win32.rs` | `tools/bindgen/tests.txt` | metadata shipped with `windows-bindgen` |

To use a new API, add its name to the matching filter list and run `cargo bindgen`. The runtime dependencies are `windows-core`, `windows-collections` and `windows-future` only; the `windows` umbrella crate is not used. Generated Win32 constants are `i32` and get converted at the use site (`.unsigned_abs()`).

`tools/bindgen` is a standalone crate with its own `Cargo.lock`, not a workspace member. It patches `windows-bindgen` to a windows-rs branch (`noexcept-static-methods`), because the SDK marks its methods `[noexcept]` and released `windows-bindgen` 0.100 generates static calls for those that do not compile. Remove the patch once the fix is upstream.

## CI and releases

`ci.yml` runs `core` (fmt, clippy and tests for `rumpus-core` and `xtask` on Linux), then `app` (clippy, tests and release build of `rumpus` on Windows), then `installer`. Every push to `main` republishes the `latest` prerelease. A tag `X.Y.Z` (no `v` prefix) publishes a GitHub release whose body is the matching `CHANGELOG.md` section; the job fails when that section is empty.

Releases are cut with `cargo release <level> --execute` from a clean `main`. Nothing goes to crates.io. All workspace members share one version. Record notable changes under `## [Unreleased]` in `CHANGELOG.md`; cargo-release stamps that heading with the version and date. The pre-release hook runs the ignored tests, so it takes over the desktop.
