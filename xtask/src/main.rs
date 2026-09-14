//! Workspace automation: build the Windows release binary and package the Inno Setup installer.
//!
//! Usage:
//!   cargo xtask build-windows [--target <triple>]
//!   cargo xtask dist [--staging <dir>]

use std::{
	path::{Path, PathBuf},
	process::{Command as Proc, ExitCode},
};

const BIN: &str = "rumpus.exe";
/// Everything the installer ships, all of it next to the executable.
const PAYLOAD: [&str; 3] = [BIN, "Windows.Devices.Midi2.dll", "Windows.Devices.Midi2.pri"];
const DEFAULT_TARGET: &str = "x86_64-pc-windows-msvc";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
enum Command {
	BuildWindows { target: String },
	Dist { staging: Option<PathBuf> },
}

fn usage() -> String {
	"usage: cargo xtask <build-windows [--target <triple>] | dist [--staging <dir>]>".to_owned()
}

fn parse(args: &[String]) -> Result<Command, String> {
	let (cmd, rest) = args.split_first().ok_or_else(usage)?;
	match cmd.as_str() {
		"build-windows" => {
			let target = parse_option(rest, "--target", "build-windows")?.unwrap_or_else(|| DEFAULT_TARGET.to_owned());
			Ok(Command::BuildWindows { target })
		}
		"dist" => Ok(Command::Dist { staging: parse_option(rest, "--staging", "dist")?.map(PathBuf::from) }),
		other => Err(format!("unknown command: {other}\n{}", usage())),
	}
}

/// Parses `rest` as at most one occurrence of `flag <value>`.
fn parse_option(rest: &[String], flag: &str, command: &str) -> Result<Option<String>, String> {
	match rest {
		[] => Ok(None),
		[f, value] if f == flag => Ok(Some(value.clone())),
		[f] if f == flag => Err(format!("{flag} requires a value")),
		[other, ..] => Err(format!("unknown {command} option: {other}")),
	}
}

/// `VersionInfoVersion` accepts only numeric x.y.z.w: strip prerelease and build metadata and pad
/// (`0.2.0-rc.1` becomes `0.2.0.0`).
fn numeric_version(version: &str) -> String {
	let core = version.split(['-', '+']).next().unwrap_or(version);
	let mut parts: Vec<u64> = core.split('.').map_while(|p| p.parse().ok()).collect();
	parts.resize(3, 0);
	format!("{}.{}.{}.0", parts[0], parts[1], parts[2])
}

/// The CPU architecture segment of a target triple (`x86_64`, `aarch64`).
fn arch_of(triple: &str) -> &str {
	triple.split('-').next().unwrap_or(triple)
}

/// The Windows name of a Rust CPU architecture, used in installer file names.
fn win_arch(rust_arch: &str) -> &'static str {
	if rust_arch == "aarch64" { "arm64" } else { "x64" }
}

/// Where a target's payload is staged for the installer.
fn staged_dir(staging: &Path, triple: &str) -> PathBuf {
	staging.join("windows").join(arch_of(triple))
}

fn stage_copy(src: &Path, dst: &Path) -> Result<(), String> {
	if let Some(parent) = dst.parent() {
		std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
	}
	std::fs::copy(src, dst).map_err(|e| format!("copy {} -> {}: {e}", src.display(), dst.display()))?;
	Ok(())
}

fn repo_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask manifest dir has a parent").to_path_buf()
}

/// Locates the Inno Setup compiler: `$ISCC`, then `PATH`, then the per-machine and per-user
/// install directories.
fn find_iscc() -> Option<PathBuf> {
	if let Some(p) = std::env::var_os("ISCC") {
		return Some(PathBuf::from(p));
	}
	let exe = "ISCC.exe";
	if let Some(paths) = std::env::var_os("PATH") {
		for dir in std::env::split_paths(&paths) {
			let candidate = dir.join(exe);
			if candidate.is_file() {
				return Some(candidate);
			}
		}
	}
	for base in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
		if let Some(root) = std::env::var_os(base) {
			let mut candidate = PathBuf::from(root);
			if base == "LOCALAPPDATA" {
				candidate.push("Programs");
			}
			candidate.push("Inno Setup 6");
			candidate.push(exe);
			if candidate.is_file() {
				return Some(candidate);
			}
		}
	}
	None
}

fn run_build_windows(target: &str) -> Result<(), String> {
	eprintln!("xtask: building rumpus (release) for {target}");
	let status = Proc::new("cargo")
		.args(["build", "--release", "--target", target, "-p", "rumpus"])
		.status()
		.map_err(|e| format!("failed to spawn cargo: {e}"))?;
	if status.success() { Ok(()) } else { Err(format!("cargo build failed for {target}")) }
}

fn run_dist(staging: Option<&Path>) -> Result<(), String> {
	let root = repo_root();
	let script = root.join("dist").join("windows").join("installer.iss");
	if !script.exists() {
		return Err(format!("installer script not found at {}", script.display()));
	}
	let staging = staging.map_or_else(|| root.join("target").join("dist"), Path::to_path_buf);
	// Passed to ISCC absolute and with native separators.
	let staging: PathBuf = std::path::absolute(&staging)
		.map_err(|e| format!("absolutize {}: {e}", staging.display()))?
		.components()
		.collect();

	// A file already present in the staging layout wins.
	let target = DEFAULT_TARGET;
	let staged = staged_dir(&staging, target);
	let built = root.join("target").join(target).join("release");
	for name in PAYLOAD {
		let dst = staged.join(name);
		if dst.is_file() {
			continue;
		}
		let src = built.join(name);
		if !src.is_file() {
			return Err(format!(
				"missing release file for {target}: {} (run: cargo xtask build-windows --target {target})",
				src.display()
			));
		}
		stage_copy(&src, &dst)?;
	}

	let outdir = root.join("target");
	let iscc = find_iscc().ok_or_else(|| "ISCC not found; install Inno Setup or set ISCC".to_owned())?;
	let arch = win_arch(arch_of(target));
	let defines = iscc_defines(VERSION, &staging, &outdir, &root.join("assets").join("icon").join("rumpus.ico"));
	eprintln!("xtask: {} {} {}", iscc.display(), defines.join(" "), script.display());
	let status = Proc::new(&iscc)
		.arg("/Q")
		.args(&defines)
		.arg(&script)
		.status()
		.map_err(|e| format!("failed to spawn ISCC: {e}"))?;
	if !status.success() {
		return Err(format!("ISCC failed for {arch}"));
	}
	eprintln!("xtask: wrote {}", outdir.join(format!("rumpus_setup-{arch}.exe")).display());
	Ok(())
}

/// The `/D` defines passed to ISCC, one argv element each. Values must not end in a backslash.
fn iscc_defines(version: &str, staging: &Path, outdir: &Path, icon: &Path) -> Vec<String> {
	vec![
		format!("/DVERSION={version}"),
		format!("/DVERSIONNUM={}", numeric_version(version)),
		format!("/DSTAGING={}", staging.display()),
		format!("/DOUTDIR={}", outdir.display()),
		format!("/DICONFILE={}", icon.display()),
	]
}

fn main() -> ExitCode {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let cmd = match parse(&args) {
		Ok(c) => c,
		Err(e) => {
			eprintln!("{e}");
			return ExitCode::FAILURE;
		}
	};
	let result = match cmd {
		Command::BuildWindows { target } => run_build_windows(&target),
		Command::Dist { staging } => run_dist(staging.as_deref()),
	};
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("xtask: {e}");
			ExitCode::FAILURE
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn args(list: &[&str]) -> Vec<String> {
		list.iter().map(|s| (*s).to_owned()).collect()
	}

	#[test]
	fn parses_commands() {
		assert_eq!(parse(&args(&["build-windows"])), Ok(Command::BuildWindows { target: DEFAULT_TARGET.to_owned() }));
		assert_eq!(
			parse(&args(&["build-windows", "--target", "aarch64-pc-windows-msvc"])),
			Ok(Command::BuildWindows { target: "aarch64-pc-windows-msvc".to_owned() })
		);
		assert_eq!(parse(&args(&["dist"])), Ok(Command::Dist { staging: None }));
		assert_eq!(
			parse(&args(&["dist", "--staging", "out"])),
			Ok(Command::Dist { staging: Some(PathBuf::from("out")) })
		);
		assert!(parse(&args(&["dist", "--staging"])).is_err());
		assert!(parse(&args(&["dist", "--bogus"])).is_err());
		assert!(parse(&args(&["nope"])).is_err());
		assert!(parse(&[]).is_err());
	}

	#[test]
	fn numeric_version_strips_prerelease_and_pads() {
		assert_eq!(numeric_version("0.1.0"), "0.1.0.0");
		assert_eq!(numeric_version("0.2.0-rc.1"), "0.2.0.0");
		assert_eq!(numeric_version("1.2.3+build.5"), "1.2.3.0");
		assert_eq!(numeric_version("1.2"), "1.2.0.0");
	}

	#[test]
	fn staged_dir_groups_by_arch() {
		assert_eq!(
			staged_dir(Path::new("stage"), "x86_64-pc-windows-msvc"),
			Path::new("stage").join("windows").join("x86_64")
		);
	}

	#[test]
	fn win_arch_maps_rust_arches() {
		assert_eq!(win_arch("x86_64"), "x64");
		assert_eq!(win_arch("aarch64"), "arm64");
	}

	#[test]
	fn iscc_defines_carry_version_and_paths() {
		assert_eq!(
			iscc_defines("1.2.3-rc.1", Path::new("stage"), Path::new("out"), Path::new("app.ico")),
			vec![
				"/DVERSION=1.2.3-rc.1".to_owned(),
				"/DVERSIONNUM=1.2.3.0".to_owned(),
				"/DSTAGING=stage".to_owned(),
				"/DOUTDIR=out".to_owned(),
				"/DICONFILE=app.ico".to_owned(),
			],
		);
	}
}
