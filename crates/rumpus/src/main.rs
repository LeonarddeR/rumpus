#![cfg_attr(not(test), windows_subsystem = "windows")]

fn main() {
	if let Err(e) = wxdragon::main(|_| {}) {
		eprintln!("rumpus: {e}");
		std::process::exit(1);
	}
}
