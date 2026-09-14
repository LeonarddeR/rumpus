//! Windows MIDI Services: availability, output enumeration and a connection that sends UMP words.

use std::{fmt, sync::OnceLock};

use rumpus_core::{
	config::OutputSelection,
	transport::{Clock, SendError, Sink},
	ump::packet_word_count,
};
use windows_core::{EventRevoker, GUID, HSTRING};

use super::bindings::Windows::Devices::Midi2::{
	Diagnostics::MidiDiagnostics,
	Enumeration::{
		Midi1PortFlow, MidiEndpointDeviceInformation, MidiEndpointDeviceInformationFilters,
		MidiEndpointDeviceInformationSortOrder,
	},
	IMidiApiStatics, MidiApi, MidiApiMode, MidiClock, MidiEndpointConnection, MidiSendMessageResults, MidiSession,
};

const US_PER_SECOND: u128 = 1_000_000;

/// Why Windows MIDI Services cannot be used on this PC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unavailable {
	/// The API is not registered on this PC; carries the activation error text.
	NotInstalled(String),
	LegacyMode,
	ServiceNotRunning,
}

impl fmt::Display for Unavailable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NotInstalled(detail) => {
				write!(f, "Windows MIDI Services is not installed on this PC ({detail}).")
			}
			Self::LegacyMode => f.write_str(
				"This PC is set to the legacy MIDI API mode. Rumpus needs Windows MIDI Services; \
				 switch the API mode in the MIDI Settings app and restart Windows.",
			),
			Self::ServiceNotRunning => f.write_str("The Windows MIDI Service could not be started."),
		}
	}
}

/// A selectable MIDI output: a MIDI 1.0 style port on an endpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputDevice {
	pub name: String,
	pub selection: OutputSelection,
}

/// Joins the multithreaded apartment; call once on every thread that touches the API.
pub fn init() -> windows_core::Result<()> {
	windows_core::init_mta()
}

/// Checks that the API is present, the PC is not in legacy mode, and the service runs.
pub fn probe() -> Result<(), Unavailable> {
	windows_core::factory::<MidiApi, IMidiApiStatics>().map_err(|e| Unavailable::NotInstalled(e.message()))?;
	if MidiApi::GetCurrentlySelectedApiMode() == MidiApiMode::LegacyMode {
		return Err(Unavailable::LegacyMode);
	}
	if MidiApi::EnsureServiceAvailable() { Ok(()) } else { Err(Unavailable::ServiceNotRunning) }
}

/// Lists output ports, one per destination entry of each endpoint's MIDI 1.0 port name table,
/// or the endpoint itself on group 1 when it publishes no table.
pub fn outputs(include_diagnostics: bool) -> Vec<OutputDevice> {
	let mut filters = MidiEndpointDeviceInformationFilters::AllStandardEndpoints;
	if include_diagnostics {
		filters |= MidiEndpointDeviceInformationFilters::DiagnosticLoopback;
	}
	let Some(endpoints) =
		MidiEndpointDeviceInformation::FindAll3(MidiEndpointDeviceInformationSortOrder::Name, filters)
	else {
		return Vec::new();
	};
	let mut outputs = Vec::new();
	for endpoint in &endpoints {
		let endpoint_id = endpoint.EndpointDeviceId().to_string_lossy();
		let endpoint_name = endpoint.Name().to_string_lossy();
		let ports: Vec<OutputDevice> = endpoint
			.GetNameTable()
			.into_iter()
			.flatten()
			.filter(|entry| entry.Flow() == Midi1PortFlow::MidiMessageDestination)
			.map(|entry| {
				let port_name = [entry.CustomName(), entry.NewStyleName(), entry.LegacyCompatibleName()]
					.into_iter()
					.map(|n| n.to_string_lossy())
					.find(|n| !n.is_empty())
					.unwrap_or_else(|| endpoint_name.clone());
				let name =
					if port_name == endpoint_name { port_name } else { format!("{port_name} ({endpoint_name})") };
				let group_index = entry.Group().map_or(0, |g| g.Index());
				OutputDevice { name, selection: OutputSelection { endpoint_id: endpoint_id.clone(), group_index } }
			})
			.collect();
		if ports.is_empty() {
			outputs
				.push(OutputDevice { name: endpoint_name, selection: OutputSelection { endpoint_id, group_index: 0 } });
		} else {
			outputs.extend(ports);
		}
	}
	outputs
}

pub fn loopback_a_id() -> String {
	MidiDiagnostics::DiagnosticsLoopbackAEndpointDeviceId().to_string_lossy()
}

pub fn loopback_b_id() -> String {
	MidiDiagnostics::DiagnosticsLoopbackBEndpointDeviceId().to_string_lossy()
}

/// The service's clock, in microseconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServiceClock;

impl Clock for ServiceClock {
	fn now_us(&self) -> u64 {
		ticks_to_us(MidiClock::Now())
	}
}

fn timestamp_frequency() -> u128 {
	static FREQUENCY: OnceLock<u64> = OnceLock::new();
	u128::from(*FREQUENCY.get_or_init(MidiClock::TimestampFrequency))
}

#[expect(clippy::cast_possible_truncation)]
fn ticks_to_us(ticks: u64) -> u64 {
	(u128::from(ticks) * US_PER_SECOND / timestamp_frequency()) as u64
}

#[expect(clippy::cast_possible_truncation)]
fn us_to_ticks(us: u64) -> u64 {
	(u128::from(us) * timestamp_frequency() / US_PER_SECOND) as u64
}

pub struct Session {
	session: MidiSession,
}

impl Session {
	pub fn open(name: &str) -> Result<Self, String> {
		let session =
			MidiSession::Create(&HSTRING::from(name)).ok_or_else(|| "Could not create a MIDI session.".to_owned())?;
		Ok(Self { session })
	}

	pub fn connect(&self, output: &OutputSelection) -> Result<Connection, String> {
		let connection = self
			.session
			.CreateEndpointConnection(&HSTRING::from(output.endpoint_id.as_str()))
			.ok_or_else(|| format!("Could not connect to MIDI endpoint {}.", output.endpoint_id))?;
		if !connection.Open() {
			return Err(format!("Could not open MIDI endpoint {}.", output.endpoint_id));
		}
		let id = connection.ConnectionId();
		Ok(Connection { session: self.session.clone(), connection, id })
	}
}

impl Drop for Session {
	fn drop(&mut self) {
		let _ = self.session.Close();
	}
}

/// An open connection to one endpoint; dropping it disconnects.
pub struct Connection {
	session: MidiSession,
	connection: MidiEndpointConnection,
	id: GUID,
}

impl Connection {
	/// Calls `handler` with the service timestamp in microseconds and the first word of every
	/// message the endpoint sends back, until the revoker is dropped.
	pub fn on_message(&self, handler: impl Fn(u64, u32) + Send + 'static) -> windows_core::Result<EventRevoker> {
		self.connection.MessageReceived(move |_, args| {
			if let Some(args) = args.as_ref() {
				handler(ticks_to_us(args.Timestamp()), args.PeekFirstWord());
			}
		})
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		self.session.DisconnectEndpointConnection(self.id);
	}
}

impl Sink for Connection {
	fn send(&mut self, at_us: u64, words: &[u32]) -> Result<(), SendError> {
		let timestamp = us_to_ticks(at_us);
		let mut rest = words;
		while let Some(&first) = rest.first() {
			let (packet, tail) = rest.split_at(packet_word_count(first).min(rest.len()));
			let result = match *packet {
				[w0] => self.connection.SendSingleMessageWords(timestamp, w0),
				[w0, w1] => self.connection.SendSingleMessageWords2(timestamp, w0, w1),
				[w0, w1, w2] => self.connection.SendSingleMessageWords3(timestamp, w0, w1, w2),
				[w0, w1, w2, w3] => self.connection.SendSingleMessageWords4(timestamp, w0, w1, w2, w3),
				_ => return Err(SendError),
			};
			if !result.contains(MidiSendMessageResults::Succeeded) {
				return Err(SendError);
			}
			rest = tail;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::{sync::mpsc, time::Duration};

	use rumpus_core::{song::ChannelMessage, ump::channel_voice};

	use super::*;

	fn loopback(id: &str) -> OutputSelection {
		OutputSelection { endpoint_id: id.to_owned(), group_index: 0 }
	}

	#[test]
	#[ignore = "needs the Windows MIDI Service"]
	fn the_service_is_available() {
		init().unwrap();
		assert_eq!(probe(), Ok(()));
	}

	#[test]
	#[ignore = "needs the Windows MIDI Service"]
	fn outputs_list_the_diagnostic_loopback_when_asked() {
		init().unwrap();
		probe().unwrap();
		let outputs = outputs(true);
		assert!(outputs.iter().any(|o| o.selection.endpoint_id == loopback_a_id()), "{outputs:?}");
		assert!(outputs.iter().all(|o| !o.name.is_empty()));
	}

	#[test]
	#[ignore = "needs the Windows MIDI Service"]
	fn words_sent_to_loopback_a_arrive_on_loopback_b_with_their_timestamp() {
		init().unwrap();
		probe().unwrap();
		let session = Session::open("Rumpus test").unwrap();
		let receiver = session.connect(&loopback(&loopback_b_id())).unwrap();
		let (tx, rx) = mpsc::channel();
		let _revoker = receiver.on_message(move |timestamp_us, word| {
			let _ = tx.send((timestamp_us, word));
		});
		let mut sender = session.connect(&loopback(&loopback_a_id())).unwrap();
		// Channel 6 keeps this note apart from other tests sharing the loopback.
		let word = channel_voice(0, 5, ChannelMessage::NoteOn { key: 61, velocity: 100 });
		let at = ServiceClock.now_us() + 200_000;
		sender.send(at, &[word]).unwrap();
		let received_at = loop {
			let (received_at, received) = rx.recv_timeout(Duration::from_secs(2)).expect("message arrives");
			if received == word {
				break received_at;
			}
		};
		// The loopback echoes the timestamp exactly; the conversion to ticks and back may lose a microsecond.
		assert!(received_at.abs_diff(at) <= 1, "sent at {at}, arrived stamped {received_at}");
	}
}
