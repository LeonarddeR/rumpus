use std::{env, fs, path::PathBuf};

use embed_manifest::{
	embed_manifest,
	manifest::{ActiveCodePage, DpiAwareness, SupportedOS::Windows10},
	new_manifest,
};

const SDK_FILES: [&str; 2] = ["Windows.Devices.Midi2.dll", "Windows.Devices.Midi2.pri"];

fn main() {
	let manifest = new_manifest("Rumpus")
		.supported_os(Windows10..=Windows10)
		.active_code_page(ActiveCodePage::Utf8)
		.dpi_awareness(DpiAwareness::PerMonitorV2);
	embed_manifest(manifest).expect("unable to embed manifest");
	embed_icon();
	copy_sdk_runtime();
}

/// Embeds the application icon and the version information shown in the executable's properties.
fn embed_icon() {
	let icon = "../../assets/icon/rumpus.ico";
	println!("cargo:rerun-if-changed={icon}");
	let mut resource = winresource::WindowsResource::new();
	resource.set_icon(icon);
	resource.set("ProductName", "Rumpus");
	resource.set("FileDescription", env!("CARGO_PKG_DESCRIPTION"));
	resource.set("LegalCopyright", "Copyright (C) 2026 Leonard de Ruijter");
	resource.compile().expect("unable to embed icon resource");
}

/// Copies the Windows MIDI Services runtime DLL next to the executables, where `WinRT` activation
/// finds it when the classes are not registered system-wide.
fn copy_sdk_runtime() {
	let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
		Ok("x86_64") => "x64",
		Ok("aarch64") => "arm64",
		Ok(other) => panic!("unsupported target architecture {other}; Windows MIDI Services needs x64 or arm64"),
		Err(e) => panic!("CARGO_CFG_TARGET_ARCH: {e}"),
	};
	let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
	let source_dir = manifest_dir.join("../../sdk/runtimes").join(format!("win-{arch}")).join("native");
	let profile_dir = profile_dir();
	// Test executables run from the deps directory, so they need their own copy.
	let target_dirs = [profile_dir.clone(), profile_dir.join("deps")];
	for name in SDK_FILES {
		let source = source_dir.join(name);
		println!("cargo:rerun-if-changed={}", source.display());
		assert!(
			source.is_file(),
			"{} is missing; run tools\\fetch-sdk.ps1 to download the Windows MIDI Services SDK",
			source.display()
		);
		for dir in &target_dirs {
			fs::copy(&source, dir.join(name)).unwrap_or_else(|e| panic!("copying {name} to {}: {e}", dir.display()));
		}
	}
}

/// The Cargo profile output directory (`target/<triple>/<profile>`), derived from `OUT_DIR`,
/// which lies three levels below it (`build/<pkg>-<hash>/out`).
fn profile_dir() -> PathBuf {
	let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
	out_dir.ancestors().nth(3).expect("OUT_DIR has fewer than three ancestors").to_path_buf()
}
