#![cfg_attr(not(test), windows_subsystem = "windows")]

mod midi;
mod player;

fn main() {
	if let Err(e) = wxdragon::main(|_| {}) {
		eprintln!("rumpus: {e}");
		std::process::exit(1);
	}
}
