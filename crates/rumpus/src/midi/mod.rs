//! Windows MIDI Services access: generated WinRT bindings and the backend built on them.

pub mod backend;
#[expect(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	clippy::all,
	clippy::pedantic,
	clippy::nursery
)]
mod bindings;
