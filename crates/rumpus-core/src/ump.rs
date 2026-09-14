//! Universal MIDI Packet words for the messages the player sends.

use midi2::{
	channel_voice1::{ChannelPressure, ControlChange, KeyPressure, NoteOff, NoteOn, PitchBend, ProgramChange},
	prelude::*,
	sysex7::Sysex7,
};

use crate::song::ChannelMessage;

const ALL_SOUND_OFF: u8 = 120;
const RESET_ALL_CONTROLLERS: u8 = 121;
const ALL_NOTES_OFF: u8 = 123;

/// Builds the single UMP word for a MIDI 1.0 channel voice message.
#[must_use]
pub fn channel_voice(group: u8, channel: u8, message: ChannelMessage) -> u32 {
	let group = u4::new(group);
	let channel = u4::new(channel);
	match message {
		ChannelMessage::NoteOff { key, velocity } => {
			let mut m = NoteOff::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_note_number(u7::new(key));
			m.set_velocity(u7::new(velocity));
			m.data()[0]
		}
		ChannelMessage::NoteOn { key, velocity } => {
			let mut m = NoteOn::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_note_number(u7::new(key));
			m.set_velocity(u7::new(velocity));
			m.data()[0]
		}
		ChannelMessage::PolyPressure { key, pressure } => {
			let mut m = KeyPressure::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_note_number(u7::new(key));
			m.set_pressure(u7::new(pressure));
			m.data()[0]
		}
		ChannelMessage::ControlChange { controller, value } => {
			let mut m = ControlChange::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_control(u7::new(controller));
			m.set_control_data(u7::new(value));
			m.data()[0]
		}
		ChannelMessage::ProgramChange { program } => {
			let mut m = ProgramChange::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_program(u7::new(program));
			m.data()[0]
		}
		ChannelMessage::ChannelPressure { pressure } => {
			let mut m = ChannelPressure::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_pressure(u7::new(pressure));
			m.data()[0]
		}
		ChannelMessage::PitchBend { value } => {
			let mut m = PitchBend::<[u32; 4]>::new();
			m.set_group(group);
			m.set_channel(channel);
			m.set_bend(u14::new(value));
			m.data()[0]
		}
	}
}

/// Packs a system exclusive payload (without `F0`/`F7`) into `SysEx7` UMP packets, two words each.
#[must_use]
pub fn sysex7(group: u8, payload: &[u8]) -> Vec<u32> {
	let mut m = Sysex7::<Vec<u32>>::new();
	m.set_payload(payload.iter().map(|&b| u7::new(b)));
	m.set_group(u4::new(group));
	m.data().to_vec()
}

/// All Sound Off, Reset All Controllers and All Notes Off for one channel.
#[must_use]
pub fn panic_words(group: u8, channel: u8) -> [u32; 3] {
	[ALL_SOUND_OFF, RESET_ALL_CONTROLLERS, ALL_NOTES_OFF]
		.map(|controller| channel_voice(group, channel, ChannelMessage::ControlChange { controller, value: 0 }))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn note_on_packs_group_channel_key_and_velocity() {
		let word = channel_voice(0, 0, ChannelMessage::NoteOn { key: 60, velocity: 100 });
		assert_eq!(word, 0x2090_3C64);
	}

	#[test]
	fn control_change_packs_group_and_channel_nibbles() {
		let word = channel_voice(0xC, 0xA, ChannelMessage::ControlChange { controller: 0x36, value: 0x37 });
		assert_eq!(word, 0x2CBA_3637);
	}

	#[test]
	fn program_change_leaves_last_byte_zero() {
		let word = channel_voice(0, 3, ChannelMessage::ProgramChange { program: 5 });
		assert_eq!(word, 0x20C3_0500);
	}

	#[test]
	fn pitch_bend_splits_into_lsb_and_msb() {
		let word = channel_voice(0, 0, ChannelMessage::PitchBend { value: 8192 });
		assert_eq!(word, 0x20E0_0040);
		let word = channel_voice(0, 0, ChannelMessage::PitchBend { value: 0x1FFF });
		assert_eq!(word, 0x20E0_7F3F);
	}

	#[test]
	fn other_channel_messages_use_their_status_nibble() {
		assert_eq!(channel_voice(1, 2, ChannelMessage::NoteOff { key: 1, velocity: 2 }), 0x2182_0102);
		assert_eq!(channel_voice(0, 0, ChannelMessage::PolyPressure { key: 7, pressure: 8 }), 0x20A0_0708);
		assert_eq!(channel_voice(0, 0, ChannelMessage::ChannelPressure { pressure: 9 }), 0x20D0_0900);
	}

	#[test]
	fn sysex7_short_payload_is_one_complete_packet() {
		assert_eq!(sysex7(0, &[1, 2, 3]), vec![0x3003_0102, 0x0300_0000]);
	}

	#[test]
	fn sysex7_empty_payload_is_one_empty_packet() {
		assert_eq!(sysex7(2, &[]), vec![0x3200_0000, 0x0000_0000]);
	}

	#[test]
	fn sysex7_long_payload_splits_into_start_and_end_packets() {
		assert_eq!(sysex7(0, &[1, 2, 3, 4, 5, 6, 7, 8]), vec![0x3016_0102, 0x0304_0506, 0x3032_0708, 0x0000_0000]);
	}

	#[test]
	fn sysex7_three_packets_use_continue_in_the_middle() {
		let payload: Vec<u8> = (1..=13).collect();
		assert_eq!(
			sysex7(0, &payload),
			vec![0x3016_0102, 0x0304_0506, 0x3026_0708, 0x090A_0B0C, 0x3031_0D00, 0x0000_0000]
		);
	}

	#[test]
	fn panic_words_silence_one_channel() {
		assert_eq!(panic_words(0, 5), [0x20B5_7800, 0x20B5_7900, 0x20B5_7B00]);
	}
}
