pub mod Windows {
	pub mod Devices {
		pub mod Midi2 {
			windows_core::imp::define_interface!(
				IMidiApiStatics,
				IMidiApiStatics_Vtbl,
				0x8087b303_0519_c0de_31d1_ee0010000000
			);
			impl windows_core::RuntimeType for IMidiApiStatics {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiApiStatics");
			}
			#[repr(C)]
			pub struct IMidiApiStatics_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub EnsureServiceAvailable:
					unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
				pub GetCurrentlySelectedApiMode:
					unsafe extern "system" fn(*mut core::ffi::c_void, *mut MidiApiMode) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiClockStatics,
				IMidiClockStatics_Vtbl,
				0x8087b303_0519_c0de_31d1_ee0010004000
			);
			impl windows_core::RuntimeType for IMidiClockStatics {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiClockStatics");
			}
			#[repr(C)]
			pub struct IMidiClockStatics_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub Now: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
				TimestampConstantSendImmediately: usize,
				TimestampConstantMessageQueueMaximumFutureTicks: usize,
				pub TimestampFrequency:
					unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiEndpointConnection,
				IMidiEndpointConnection_Vtbl,
				0x8087b303_0519_c0de_31d1_dd0010006000
			);
			impl windows_core::RuntimeType for IMidiEndpointConnection {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiEndpointConnection");
			}
			#[repr(C)]
			pub struct IMidiEndpointConnection_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				LogMessageDataValidationErrorDetails: usize,
				SetLogMessageDataValidationErrorDetails: usize,
				pub Open: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
				MessageProcessingPlugins: usize,
				AddMessageProcessingPlugin: usize,
				RemoveMessageProcessingPlugin: usize,
				SendSingleMessagePacket: usize,
				SendSingleMessageStruct: usize,
				SendSingleMessageWordArray: usize,
				pub SendSingleMessageWords: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					u64,
					u32,
					*mut MidiSendMessageResults,
				) -> windows_core::HRESULT,
				pub SendSingleMessageWords2: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					u64,
					u32,
					u32,
					*mut MidiSendMessageResults,
				) -> windows_core::HRESULT,
				pub SendSingleMessageWords3: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					u64,
					u32,
					u32,
					u32,
					*mut MidiSendMessageResults,
				) -> windows_core::HRESULT,
				pub SendSingleMessageWords4: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					u64,
					u32,
					u32,
					u32,
					u32,
					*mut MidiSendMessageResults,
				) -> windows_core::HRESULT,
				SendSingleMessageBuffer: usize,
				pub SendMultipleMessagesWordList: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					u64,
					*mut core::ffi::c_void,
					*mut MidiSendMessageResults,
				) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiEndpointConnectionSettings,
				IMidiEndpointConnectionSettings_Vtbl,
				0x8087b303_0519_c0de_31d1_dd0010007000
			);
			impl windows_core::RuntimeType for IMidiEndpointConnectionSettings {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
					b"Windows.Devices.Midi2.IMidiEndpointConnectionSettings",
				);
			}
			#[repr(C)]
			pub struct IMidiEndpointConnectionSettings_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
			}
			windows_core::imp::define_interface!(
				IMidiEndpointConnectionSource,
				IMidiEndpointConnectionSource_Vtbl,
				0x8087b303_0519_c0de_31d1_cc001000f030
			);
			impl windows_core::RuntimeType for IMidiEndpointConnectionSource {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiEndpointConnectionSource");
			}
			windows_core::imp::interface_hierarchy!(
				IMidiEndpointConnectionSource,
				windows_core::IUnknown,
				windows_core::IInspectable
			);
			impl IMidiEndpointConnectionSource {
				pub fn ConnectionId(&self) -> windows_core::GUID {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).ConnectionId)(
							windows_core::Interface::as_raw(self),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
			}
			impl windows_core::RuntimeName for IMidiEndpointConnectionSource {
				const NAME: &'static str = "Windows.Devices.Midi2.IMidiEndpointConnectionSource";
			}
			#[repr(C)]
			pub struct IMidiEndpointConnectionSource_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				EndpointDeviceDisconnected: usize,
				RemoveEndpointDeviceDisconnected: usize,
				EndpointDeviceReconnected: usize,
				RemoveEndpointDeviceReconnected: usize,
				Tag: usize,
				SetTag: usize,
				pub ConnectionId:
					unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_core::GUID) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(IMidiGroup, IMidiGroup_Vtbl, 0x8087b303_0519_c0de_31d1_dd0010002000);
			impl windows_core::RuntimeType for IMidiGroup {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiGroup");
			}
			#[repr(C)]
			pub struct IMidiGroup_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub Index: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u8) -> windows_core::HRESULT,
				pub SetIndex: unsafe extern "system" fn(*mut core::ffi::c_void, u8) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiMessageReceivedEventArgs,
				IMidiMessageReceivedEventArgs_Vtbl,
				0x8087b303_0519_c0de_31d1_dd0010008000
			);
			impl windows_core::RuntimeType for IMidiMessageReceivedEventArgs {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiMessageReceivedEventArgs");
			}
			#[repr(C)]
			pub struct IMidiMessageReceivedEventArgs_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub Timestamp: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
				PacketType: usize,
				MessageType: usize,
				pub PeekFirstWord: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiMessageReceivedEventSource,
				IMidiMessageReceivedEventSource_Vtbl,
				0x8087b303_0519_c0de_31d1_cc001000f050
			);
			impl windows_core::RuntimeType for IMidiMessageReceivedEventSource {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
					b"Windows.Devices.Midi2.IMidiMessageReceivedEventSource",
				);
			}
			windows_core::imp::interface_hierarchy!(
				IMidiMessageReceivedEventSource,
				windows_core::IUnknown,
				windows_core::IInspectable
			);
			impl IMidiMessageReceivedEventSource {
				pub fn MessageReceived<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
				where
					F: Fn(windows_core::Ref<Self>, windows_core::Ref<MidiMessageReceivedEventArgs>) + Send + 'static,
				{
					let handler =
						<super::super::Foundation::TypedEventHandler<Self, MidiMessageReceivedEventArgs>>::new(
							move |a0, a1| {
								handler(a0, a1);
								Ok(())
							},
						);
					unsafe {
						let mut result__ = core::mem::zeroed();
						let token__ = (windows_core::Interface::vtable(self).MessageReceived)(
							windows_core::Interface::as_raw(self),
							windows_core::Interface::as_raw(&handler),
							&mut result__,
						)
						.map(|| result__)?;
						Ok(windows_core::EventRevoker::new(
							self.clone(),
							token__,
							windows_core::Interface::vtable(self).RemoveMessageReceived,
						))
					}
				}
			}
			impl windows_core::RuntimeName for IMidiMessageReceivedEventSource {
				const NAME: &'static str = "Windows.Devices.Midi2.IMidiMessageReceivedEventSource";
			}
			#[repr(C)]
			pub struct IMidiMessageReceivedEventSource_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub MessageReceived: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					*mut core::ffi::c_void,
					*mut i64,
				) -> windows_core::HRESULT,
				pub RemoveMessageReceived:
					unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiSession,
				IMidiSession_Vtbl,
				0x8087b303_0519_c0de_31d1_dd0010009000
			);
			impl windows_core::RuntimeType for IMidiSession {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiSession");
			}
			#[repr(C)]
			pub struct IMidiSession_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				SessionId: usize,
				Name: usize,
				IsOpen: usize,
				Connections: usize,
				pub CreateEndpointConnection: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					*mut core::ffi::c_void,
					*mut *mut core::ffi::c_void,
				) -> windows_core::HRESULT,
				pub CreateEndpointConnection2: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					*mut core::ffi::c_void,
					*mut core::ffi::c_void,
					*mut *mut core::ffi::c_void,
				) -> windows_core::HRESULT,
				pub DisconnectEndpointConnection:
					unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::GUID) -> windows_core::HRESULT,
			}
			windows_core::imp::define_interface!(
				IMidiSessionStatics,
				IMidiSessionStatics_Vtbl,
				0x8087b303_0519_c0de_31d1_ee0010009000
			);
			impl windows_core::RuntimeType for IMidiSessionStatics {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_interface::<Self>();
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.IMidiSessionStatics");
			}
			#[repr(C)]
			pub struct IMidiSessionStatics_Vtbl {
				pub base__: windows_core::IInspectable_Vtbl,
				pub Create: unsafe extern "system" fn(
					*mut core::ffi::c_void,
					*mut core::ffi::c_void,
					*mut *mut core::ffi::c_void,
				) -> windows_core::HRESULT,
			}
			pub struct MidiApi;
			impl MidiApi {
				pub fn EnsureServiceAvailable() -> bool {
					Self::IMidiApiStatics(|this| unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).EnsureServiceAvailable)(
							windows_core::Interface::as_raw(this),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					})
				}
				pub fn GetCurrentlySelectedApiMode() -> MidiApiMode {
					Self::IMidiApiStatics(|this| unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).GetCurrentlySelectedApiMode)(
							windows_core::Interface::as_raw(this),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					})
				}
				fn IMidiApiStatics<R, F: FnOnce(&IMidiApiStatics) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiApi, IMidiApiStatics> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeName for MidiApi {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiApi";
			}
			#[repr(transparent)]
			#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
			pub struct MidiApiMode(pub i32);
			impl MidiApiMode {
				pub const FullWindowsMidiServicesMode: Self = Self(0);
				pub const LegacyMode: Self = Self(1);
				pub const HybridLegacyMode: Self = Self(2);
			}
			impl windows_core::imp::TypeKind for MidiApiMode {
				type TypeKind = windows_core::imp::CopyType;
			}
			impl windows_core::RuntimeType for MidiApiMode {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Devices.Midi2.MidiApiMode;i4)");
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.MidiApiMode");
			}
			pub struct MidiClock;
			impl MidiClock {
				pub fn Now() -> u64 {
					Self::IMidiClockStatics(|this| unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).Now)(
							windows_core::Interface::as_raw(this),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					})
				}
				pub fn TimestampFrequency() -> u64 {
					Self::IMidiClockStatics(|this| unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).TimestampFrequency)(
							windows_core::Interface::as_raw(this),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					})
				}
				fn IMidiClockStatics<R, F: FnOnce(&IMidiClockStatics) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiClock, IMidiClockStatics> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeName for MidiClock {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiClock";
			}
			#[repr(transparent)]
			#[derive(Clone, Debug, Eq, PartialEq)]
			pub struct MidiEndpointConnection(windows_core::IUnknown);
			windows_core::imp::interface_hierarchy!(
				MidiEndpointConnection,
				windows_core::IUnknown,
				windows_core::IInspectable
			);
			windows_core::imp::required_hierarchy!(
				MidiEndpointConnection,
				IMidiEndpointConnectionSource,
				IMidiMessageReceivedEventSource
			);
			impl MidiEndpointConnection {
				pub fn Open(&self) -> bool {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).Open)(
							windows_core::Interface::as_raw(self),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SendSingleMessageWords(&self, timestamp: u64, word0: u32) -> MidiSendMessageResults {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).SendSingleMessageWords)(
							windows_core::Interface::as_raw(self),
							timestamp,
							word0,
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SendSingleMessageWords2(
					&self,
					timestamp: u64,
					word0: u32,
					word1: u32,
				) -> MidiSendMessageResults {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).SendSingleMessageWords2)(
							windows_core::Interface::as_raw(self),
							timestamp,
							word0,
							word1,
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SendSingleMessageWords3(
					&self,
					timestamp: u64,
					word0: u32,
					word1: u32,
					word2: u32,
				) -> MidiSendMessageResults {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).SendSingleMessageWords3)(
							windows_core::Interface::as_raw(self),
							timestamp,
							word0,
							word1,
							word2,
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SendSingleMessageWords4(
					&self,
					timestamp: u64,
					word0: u32,
					word1: u32,
					word2: u32,
					word3: u32,
				) -> MidiSendMessageResults {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).SendSingleMessageWords4)(
							windows_core::Interface::as_raw(self),
							timestamp,
							word0,
							word1,
							word2,
							word3,
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SendMultipleMessagesWordList<P1>(&self, timestamp: u64, words: P1) -> MidiSendMessageResults
				where
					P1: windows_core::Param<windows_collections::IIterable<u32>>,
				{
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).SendMultipleMessagesWordList)(
							windows_core::Interface::as_raw(self),
							timestamp,
							words.param().abi(),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn ConnectionId(&self) -> windows_core::GUID {
					let this = &windows_core::Interface::cast::<IMidiEndpointConnectionSource>(self).unwrap();
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).ConnectionId)(
							windows_core::Interface::as_raw(this),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn MessageReceived<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
				where
					F: Fn(
							windows_core::Ref<IMidiMessageReceivedEventSource>,
							windows_core::Ref<MidiMessageReceivedEventArgs>,
						) + Send
						+ 'static,
				{
					let this = &windows_core::Interface::cast::<IMidiMessageReceivedEventSource>(self)?;
					let handler = <super::super::Foundation::TypedEventHandler<
						IMidiMessageReceivedEventSource,
						MidiMessageReceivedEventArgs,
					>>::new(move |a0, a1| {
						handler(a0, a1);
						Ok(())
					});
					unsafe {
						let mut result__ = core::mem::zeroed();
						let token__ = (windows_core::Interface::vtable(this).MessageReceived)(
							windows_core::Interface::as_raw(this),
							windows_core::Interface::as_raw(&handler),
							&mut result__,
						)
						.map(|| result__)?;
						Ok(windows_core::EventRevoker::new(
							this.clone(),
							token__,
							windows_core::Interface::vtable(this).RemoveMessageReceived,
						))
					}
				}
				fn IMidiEndpointConnectionStatics<
					R,
					F: FnOnce(&IMidiEndpointConnectionStatics) -> windows_core::Result<R>,
				>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<
						MidiEndpointConnection,
						IMidiEndpointConnectionStatics,
					> = windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeType for MidiEndpointConnection {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_class::<Self, IMidiEndpointConnection>();
			}
			unsafe impl windows_core::Interface for MidiEndpointConnection {
				type Vtable = <IMidiEndpointConnection as windows_core::Interface>::Vtable;
				const IID: windows_core::GUID = <IMidiEndpointConnection as windows_core::Interface>::IID;
			}
			impl windows_core::RuntimeName for MidiEndpointConnection {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiEndpointConnection";
			}
			unsafe impl Send for MidiEndpointConnection {}
			unsafe impl Sync for MidiEndpointConnection {}
			#[repr(transparent)]
			#[derive(Clone, Debug, Eq, PartialEq)]
			pub struct MidiEndpointConnectionSettings(windows_core::IUnknown);
			windows_core::imp::interface_hierarchy!(
				MidiEndpointConnectionSettings,
				windows_core::IUnknown,
				windows_core::IInspectable
			);
			impl MidiEndpointConnectionSettings {
				pub fn new() -> windows_core::Result<Self> {
					Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
				}
				fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<
						MidiEndpointConnectionSettings,
						windows_core::imp::IGenericFactory,
					> = windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
				fn IMidiEndpointConnectionSettingsFactory<
					R,
					F: FnOnce(&IMidiEndpointConnectionSettingsFactory) -> windows_core::Result<R>,
				>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<
						MidiEndpointConnectionSettings,
						IMidiEndpointConnectionSettingsFactory,
					> = windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeType for MidiEndpointConnectionSettings {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_class::<Self, IMidiEndpointConnectionSettings>();
			}
			unsafe impl windows_core::Interface for MidiEndpointConnectionSettings {
				type Vtable = <IMidiEndpointConnectionSettings as windows_core::Interface>::Vtable;
				const IID: windows_core::GUID = <IMidiEndpointConnectionSettings as windows_core::Interface>::IID;
			}
			impl windows_core::RuntimeName for MidiEndpointConnectionSettings {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiEndpointConnectionSettings";
			}
			unsafe impl Send for MidiEndpointConnectionSettings {}
			unsafe impl Sync for MidiEndpointConnectionSettings {}
			#[repr(transparent)]
			#[derive(Clone, Debug, Eq, PartialEq)]
			pub struct MidiGroup(windows_core::IUnknown);
			windows_core::imp::interface_hierarchy!(MidiGroup, windows_core::IUnknown, windows_core::IInspectable);
			impl MidiGroup {
				pub fn new() -> windows_core::Result<Self> {
					Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
				}
				fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiGroup, windows_core::imp::IGenericFactory> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
				pub fn Index(&self) -> u8 {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).Index)(
							windows_core::Interface::as_raw(self),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn SetIndex(&self, value: u8) {
					unsafe {
						let hresult__ = (windows_core::Interface::vtable(self).SetIndex)(
							windows_core::Interface::as_raw(self),
							value,
						);
						debug_assert!(hresult__.0 == 0);
					}
				}
				fn IMidiGroupFactory<R, F: FnOnce(&IMidiGroupFactory) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiGroup, IMidiGroupFactory> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
				fn IMidiGroupStatics<R, F: FnOnce(&IMidiGroupStatics) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiGroup, IMidiGroupStatics> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeType for MidiGroup {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_class::<Self, IMidiGroup>();
			}
			unsafe impl windows_core::Interface for MidiGroup {
				type Vtable = <IMidiGroup as windows_core::Interface>::Vtable;
				const IID: windows_core::GUID = <IMidiGroup as windows_core::Interface>::IID;
			}
			impl windows_core::RuntimeName for MidiGroup {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiGroup";
			}
			unsafe impl Send for MidiGroup {}
			unsafe impl Sync for MidiGroup {}
			#[repr(transparent)]
			#[derive(Clone, Debug, Eq, PartialEq)]
			pub struct MidiMessageReceivedEventArgs(windows_core::IUnknown);
			windows_core::imp::interface_hierarchy!(
				MidiMessageReceivedEventArgs,
				windows_core::IUnknown,
				windows_core::IInspectable
			);
			impl MidiMessageReceivedEventArgs {
				pub fn Timestamp(&self) -> u64 {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).Timestamp)(
							windows_core::Interface::as_raw(self),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
				pub fn PeekFirstWord(&self) -> u32 {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).PeekFirstWord)(
							windows_core::Interface::as_raw(self),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						result__
					}
				}
			}
			impl windows_core::RuntimeType for MidiMessageReceivedEventArgs {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_class::<Self, IMidiMessageReceivedEventArgs>();
			}
			unsafe impl windows_core::Interface for MidiMessageReceivedEventArgs {
				type Vtable = <IMidiMessageReceivedEventArgs as windows_core::Interface>::Vtable;
				const IID: windows_core::GUID = <IMidiMessageReceivedEventArgs as windows_core::Interface>::IID;
			}
			impl windows_core::RuntimeName for MidiMessageReceivedEventArgs {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiMessageReceivedEventArgs";
			}
			unsafe impl Send for MidiMessageReceivedEventArgs {}
			unsafe impl Sync for MidiMessageReceivedEventArgs {}
			#[repr(transparent)]
			#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
			pub struct MidiSendMessageResults(pub u32);
			impl MidiSendMessageResults {
				pub const Succeeded: Self = Self(2147483648);
				pub const Failed: Self = Self(268435456);
				pub const BufferFull: Self = Self(65536);
				pub const EndpointConnectionClosedOrInvalid: Self = Self(262144);
				pub const InvalidMessageTypeForWordCount: Self = Self(1048576);
				pub const InvalidMessageOther: Self = Self(2097152);
				pub const DataIndexOutOfRange: Self = Self(4194304);
				pub const TimestampOutOfRange: Self = Self(8388608);
				pub const TransmissionWordCountExceeded: Self = Self(16777216);
			}
			impl windows_core::imp::TypeKind for MidiSendMessageResults {
				type TypeKind = windows_core::imp::CopyType;
			}
			impl windows_core::RuntimeType for MidiSendMessageResults {
				const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
					b"enum(Windows.Devices.Midi2.MidiSendMessageResults;u4)",
				);
				const NAME: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.MidiSendMessageResults");
			}
			impl MidiSendMessageResults {
				pub const fn contains(&self, other: Self) -> bool {
					self.0 & other.0 == other.0
				}
			}
			impl core::ops::BitOr for MidiSendMessageResults {
				type Output = Self;
				fn bitor(self, other: Self) -> Self {
					Self(self.0 | other.0)
				}
			}
			impl core::ops::BitAnd for MidiSendMessageResults {
				type Output = Self;
				fn bitand(self, other: Self) -> Self {
					Self(self.0 & other.0)
				}
			}
			impl core::ops::BitOrAssign for MidiSendMessageResults {
				fn bitor_assign(&mut self, other: Self) {
					self.0.bitor_assign(other.0);
				}
			}
			impl core::ops::BitAndAssign for MidiSendMessageResults {
				fn bitand_assign(&mut self, other: Self) {
					self.0.bitand_assign(other.0);
				}
			}
			impl core::ops::Not for MidiSendMessageResults {
				type Output = Self;
				fn not(self) -> Self {
					Self(self.0.not())
				}
			}
			#[repr(transparent)]
			#[derive(Clone, Debug, Eq, PartialEq)]
			pub struct MidiSession(windows_core::IUnknown);
			windows_core::imp::interface_hierarchy!(MidiSession, windows_core::IUnknown, windows_core::IInspectable);
			windows_core::imp::required_hierarchy!(MidiSession, super::super::Foundation::IClosable);
			impl MidiSession {
				pub fn Close(&self) -> windows_core::Result<()> {
					let this = &windows_core::Interface::cast::<super::super::Foundation::IClosable>(self)?;
					unsafe { (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this)).ok() }
				}
				pub fn CreateEndpointConnection(
					&self,
					endpointdeviceid: &windows_core::HSTRING,
				) -> Option<MidiEndpointConnection> {
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).CreateEndpointConnection)(
							windows_core::Interface::as_raw(self),
							core::mem::transmute_copy(endpointdeviceid),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						core::mem::transmute(result__)
					}
				}
				pub fn CreateEndpointConnection2<P1>(
					&self,
					endpointdeviceid: &windows_core::HSTRING,
					settings: P1,
				) -> Option<MidiEndpointConnection>
				where
					P1: windows_core::Param<MidiEndpointConnectionSettings>,
				{
					unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(self).CreateEndpointConnection2)(
							windows_core::Interface::as_raw(self),
							core::mem::transmute_copy(endpointdeviceid),
							settings.param().abi(),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						core::mem::transmute(result__)
					}
				}
				pub fn DisconnectEndpointConnection(&self, endpointconnectionid: windows_core::GUID) {
					unsafe {
						let hresult__ = (windows_core::Interface::vtable(self).DisconnectEndpointConnection)(
							windows_core::Interface::as_raw(self),
							endpointconnectionid,
						);
						debug_assert!(hresult__.0 == 0);
					}
				}
				pub fn Create(sessionname: &windows_core::HSTRING) -> Option<Self> {
					Self::IMidiSessionStatics(|this| unsafe {
						let mut result__ = core::mem::zeroed();
						let hresult__ = (windows_core::Interface::vtable(this).Create)(
							windows_core::Interface::as_raw(this),
							core::mem::transmute_copy(sessionname),
							&mut result__,
						);
						debug_assert!(hresult__.0 == 0);
						core::mem::transmute(result__)
					})
				}
				fn IMidiSessionStatics<R, F: FnOnce(&IMidiSessionStatics) -> windows_core::Result<R>>(
					callback: F,
				) -> windows_core::Result<R> {
					static SHARED: windows_core::imp::FactoryCache<MidiSession, IMidiSessionStatics> =
						windows_core::imp::FactoryCache::new();
					SHARED.call(callback)
				}
			}
			impl windows_core::RuntimeType for MidiSession {
				const SIGNATURE: windows_core::imp::ConstBuffer =
					windows_core::imp::ConstBuffer::for_class::<Self, IMidiSession>();
			}
			unsafe impl windows_core::Interface for MidiSession {
				type Vtable = <IMidiSession as windows_core::Interface>::Vtable;
				const IID: windows_core::GUID = <IMidiSession as windows_core::Interface>::IID;
			}
			impl windows_core::RuntimeName for MidiSession {
				const NAME: &'static str = "Windows.Devices.Midi2.MidiSession";
			}
			unsafe impl Send for MidiSession {}
			unsafe impl Sync for MidiSession {}
			pub mod Diagnostics {
				windows_core::imp::define_interface!(
					IMidiDiagnosticsStatics,
					IMidiDiagnosticsStatics_Vtbl,
					0x8087b303_0519_c0de_31d1_ee0030001000
				);
				impl windows_core::RuntimeType for IMidiDiagnosticsStatics {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_interface::<Self>();
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Diagnostics.IMidiDiagnosticsStatics",
					);
				}
				#[repr(C)]
				pub struct IMidiDiagnosticsStatics_Vtbl {
					pub base__: windows_core::IInspectable_Vtbl,
					pub DiagnosticsLoopbackAEndpointDeviceId: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub DiagnosticsLoopbackBEndpointDeviceId: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
				}
				pub struct MidiDiagnostics;
				impl MidiDiagnostics {
					pub fn DiagnosticsLoopbackAEndpointDeviceId() -> windows_core::HSTRING {
						Self::IMidiDiagnosticsStatics(|this| unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(this)
								.DiagnosticsLoopbackAEndpointDeviceId)(
								windows_core::Interface::as_raw(this), &mut result__
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						})
					}
					pub fn DiagnosticsLoopbackBEndpointDeviceId() -> windows_core::HSTRING {
						Self::IMidiDiagnosticsStatics(|this| unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(this)
								.DiagnosticsLoopbackBEndpointDeviceId)(
								windows_core::Interface::as_raw(this), &mut result__
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						})
					}
					fn IMidiDiagnosticsStatics<R, F: FnOnce(&IMidiDiagnosticsStatics) -> windows_core::Result<R>>(
						callback: F,
					) -> windows_core::Result<R> {
						static SHARED: windows_core::imp::FactoryCache<MidiDiagnostics, IMidiDiagnosticsStatics> =
							windows_core::imp::FactoryCache::new();
						SHARED.call(callback)
					}
				}
				impl windows_core::RuntimeName for MidiDiagnostics {
					const NAME: &'static str = "Windows.Devices.Midi2.Diagnostics.MidiDiagnostics";
				}
			}
			pub mod Enumeration {
				windows_core::imp::define_interface!(
					IMidi1PortNameTableEntry,
					IMidi1PortNameTableEntry_Vtbl,
					0x8087b303_0519_c0de_31d1_dd0040016000
				);
				impl windows_core::RuntimeType for IMidi1PortNameTableEntry {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_interface::<Self>();
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Enumeration.IMidi1PortNameTableEntry",
					);
				}
				#[repr(C)]
				pub struct IMidi1PortNameTableEntry_Vtbl {
					pub base__: windows_core::IInspectable_Vtbl,
					pub Group: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub Flow:
						unsafe extern "system" fn(*mut core::ffi::c_void, *mut Midi1PortFlow) -> windows_core::HRESULT,
					pub CustomName: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub LegacyCompatibleName: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub NewStyleName: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
				}
				windows_core::imp::define_interface!(
					IMidiEndpointDeviceInformation,
					IMidiEndpointDeviceInformation_Vtbl,
					0x8087b303_0519_c0de_31d1_dd004000a000
				);
				impl windows_core::RuntimeType for IMidiEndpointDeviceInformation {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_interface::<Self>();
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Enumeration.IMidiEndpointDeviceInformation",
					);
				}
				#[repr(C)]
				pub struct IMidiEndpointDeviceInformation_Vtbl {
					pub base__: windows_core::IInspectable_Vtbl,
					pub EndpointDeviceId: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub Name: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					ContainerId: usize,
					DeviceInstanceId: usize,
					EndpointPurpose: usize,
					GetDeclaredEndpointInfo: usize,
					DeclaredEndpointInfoLastUpdateTime: usize,
					GetDeclaredDeviceIdentity: usize,
					DeclaredDeviceIdentityLastUpdateTime: usize,
					GetDeclaredStreamConfiguration: usize,
					DeclaredStreamConfigurationLastUpdateTime: usize,
					GetDeclaredFunctionBlocks: usize,
					DeclaredFunctionBlocksLastUpdateTime: usize,
					GetGroupTerminalBlocks: usize,
					GetUserSuppliedInfo: usize,
					GetTransportSuppliedInfo: usize,
					ParentDeviceInstanceId: usize,
					GetParentDeviceInformation: usize,
					GetContainerDeviceInformation: usize,
					Properties: usize,
					pub GetNameTable: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
				}
				windows_core::imp::define_interface!(
					IMidiEndpointDeviceInformationStatics,
					IMidiEndpointDeviceInformationStatics_Vtbl,
					0x8087b303_0519_c0de_31d1_ee004000a000
				);
				impl windows_core::RuntimeType for IMidiEndpointDeviceInformationStatics {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_interface::<Self>();
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Enumeration.IMidiEndpointDeviceInformationStatics",
					);
				}
				#[repr(C)]
				pub struct IMidiEndpointDeviceInformationStatics_Vtbl {
					pub base__: windows_core::IInspectable_Vtbl,
					CreateFromEndpointDeviceId: usize,
					pub FindAll: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub FindAll2: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						MidiEndpointDeviceInformationSortOrder,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
					pub FindAll3: unsafe extern "system" fn(
						*mut core::ffi::c_void,
						MidiEndpointDeviceInformationSortOrder,
						MidiEndpointDeviceInformationFilters,
						*mut *mut core::ffi::c_void,
					) -> windows_core::HRESULT,
				}
				#[repr(transparent)]
				#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
				pub struct Midi1PortFlow(pub i32);
				impl Midi1PortFlow {
					pub const MidiMessageSource: Self = Self(0);
					pub const MidiMessageDestination: Self = Self(1);
				}
				impl windows_core::imp::TypeKind for Midi1PortFlow {
					type TypeKind = windows_core::imp::CopyType;
				}
				impl windows_core::RuntimeType for Midi1PortFlow {
					const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"enum(Windows.Devices.Midi2.Enumeration.Midi1PortFlow;i4)",
					);
					const NAME: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::from_slice(b"Windows.Devices.Midi2.Enumeration.Midi1PortFlow");
				}
				#[repr(transparent)]
				#[derive(Clone, Debug, Eq, PartialEq)]
				pub struct Midi1PortNameTableEntry(windows_core::IUnknown);
				windows_core::imp::interface_hierarchy!(
					Midi1PortNameTableEntry,
					windows_core::IUnknown,
					windows_core::IInspectable
				);
				impl Midi1PortNameTableEntry {
					pub fn Group(&self) -> Option<super::MidiGroup> {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).Group)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn Flow(&self) -> Midi1PortFlow {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).Flow)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							result__
						}
					}
					pub fn CustomName(&self) -> windows_core::HSTRING {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).CustomName)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn LegacyCompatibleName(&self) -> windows_core::HSTRING {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).LegacyCompatibleName)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn NewStyleName(&self) -> windows_core::HSTRING {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).NewStyleName)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
				}
				impl windows_core::RuntimeType for Midi1PortNameTableEntry {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_class::<Self, IMidi1PortNameTableEntry>();
				}
				unsafe impl windows_core::Interface for Midi1PortNameTableEntry {
					type Vtable = <IMidi1PortNameTableEntry as windows_core::Interface>::Vtable;
					const IID: windows_core::GUID = <IMidi1PortNameTableEntry as windows_core::Interface>::IID;
				}
				impl windows_core::RuntimeName for Midi1PortNameTableEntry {
					const NAME: &'static str = "Windows.Devices.Midi2.Enumeration.Midi1PortNameTableEntry";
				}
				unsafe impl Send for Midi1PortNameTableEntry {}
				unsafe impl Sync for Midi1PortNameTableEntry {}
				#[repr(transparent)]
				#[derive(Clone, Debug, Eq, PartialEq)]
				pub struct MidiEndpointDeviceInformation(windows_core::IUnknown);
				windows_core::imp::interface_hierarchy!(
					MidiEndpointDeviceInformation,
					windows_core::IUnknown,
					windows_core::IInspectable
				);
				impl MidiEndpointDeviceInformation {
					pub fn EndpointDeviceId(&self) -> windows_core::HSTRING {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).EndpointDeviceId)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn Name(&self) -> windows_core::HSTRING {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).Name)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn GetNameTable(&self) -> Option<windows_collections::IVectorView<Midi1PortNameTableEntry>> {
						unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(self).GetNameTable)(
								windows_core::Interface::as_raw(self),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						}
					}
					pub fn FindAll() -> Option<windows_collections::IVectorView<Self>> {
						Self::IMidiEndpointDeviceInformationStatics(|this| unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(this).FindAll)(
								windows_core::Interface::as_raw(this),
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						})
					}
					pub fn FindAll2(
						sortorder: MidiEndpointDeviceInformationSortOrder,
					) -> Option<windows_collections::IVectorView<Self>> {
						Self::IMidiEndpointDeviceInformationStatics(|this| unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(this).FindAll2)(
								windows_core::Interface::as_raw(this),
								sortorder,
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						})
					}
					pub fn FindAll3(
						sortorder: MidiEndpointDeviceInformationSortOrder,
						endpointtypestoinclude: MidiEndpointDeviceInformationFilters,
					) -> Option<windows_collections::IVectorView<Self>> {
						Self::IMidiEndpointDeviceInformationStatics(|this| unsafe {
							let mut result__ = core::mem::zeroed();
							let hresult__ = (windows_core::Interface::vtable(this).FindAll3)(
								windows_core::Interface::as_raw(this),
								sortorder,
								endpointtypestoinclude,
								&mut result__,
							);
							debug_assert!(hresult__.0 == 0);
							core::mem::transmute(result__)
						})
					}
					fn IMidiEndpointDeviceInformationStatics<
						R,
						F: FnOnce(&IMidiEndpointDeviceInformationStatics) -> windows_core::Result<R>,
					>(
						callback: F,
					) -> windows_core::Result<R> {
						static SHARED: windows_core::imp::FactoryCache<
							MidiEndpointDeviceInformation,
							IMidiEndpointDeviceInformationStatics,
						> = windows_core::imp::FactoryCache::new();
						SHARED.call(callback)
					}
				}
				impl windows_core::RuntimeType for MidiEndpointDeviceInformation {
					const SIGNATURE: windows_core::imp::ConstBuffer =
						windows_core::imp::ConstBuffer::for_class::<Self, IMidiEndpointDeviceInformation>();
				}
				unsafe impl windows_core::Interface for MidiEndpointDeviceInformation {
					type Vtable = <IMidiEndpointDeviceInformation as windows_core::Interface>::Vtable;
					const IID: windows_core::GUID = <IMidiEndpointDeviceInformation as windows_core::Interface>::IID;
				}
				impl windows_core::RuntimeName for MidiEndpointDeviceInformation {
					const NAME: &'static str = "Windows.Devices.Midi2.Enumeration.MidiEndpointDeviceInformation";
				}
				unsafe impl Send for MidiEndpointDeviceInformation {}
				unsafe impl Sync for MidiEndpointDeviceInformation {}
				#[repr(transparent)]
				#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
				pub struct MidiEndpointDeviceInformationFilters(pub u32);
				impl MidiEndpointDeviceInformationFilters {
					pub const StandardNativeUniversalMidiPacketFormat: Self = Self(1);
					pub const StandardNativeMidi1ByteFormat: Self = Self(2);
					pub const VirtualDeviceResponder: Self = Self(256);
					pub const DiagnosticLoopback: Self = Self(65536);
					pub const DiagnosticPing: Self = Self(131072);
					pub const AllStandardEndpoints: Self = Self(3);
				}
				impl windows_core::imp::TypeKind for MidiEndpointDeviceInformationFilters {
					type TypeKind = windows_core::imp::CopyType;
				}
				impl windows_core::RuntimeType for MidiEndpointDeviceInformationFilters {
					const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"enum(Windows.Devices.Midi2.Enumeration.MidiEndpointDeviceInformationFilters;u4)",
					);
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Enumeration.MidiEndpointDeviceInformationFilters",
					);
				}
				impl MidiEndpointDeviceInformationFilters {
					pub const fn contains(&self, other: Self) -> bool {
						self.0 & other.0 == other.0
					}
				}
				impl core::ops::BitOr for MidiEndpointDeviceInformationFilters {
					type Output = Self;
					fn bitor(self, other: Self) -> Self {
						Self(self.0 | other.0)
					}
				}
				impl core::ops::BitAnd for MidiEndpointDeviceInformationFilters {
					type Output = Self;
					fn bitand(self, other: Self) -> Self {
						Self(self.0 & other.0)
					}
				}
				impl core::ops::BitOrAssign for MidiEndpointDeviceInformationFilters {
					fn bitor_assign(&mut self, other: Self) {
						self.0.bitor_assign(other.0);
					}
				}
				impl core::ops::BitAndAssign for MidiEndpointDeviceInformationFilters {
					fn bitand_assign(&mut self, other: Self) {
						self.0.bitand_assign(other.0);
					}
				}
				impl core::ops::Not for MidiEndpointDeviceInformationFilters {
					type Output = Self;
					fn not(self) -> Self {
						Self(self.0.not())
					}
				}
				#[repr(transparent)]
				#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
				pub struct MidiEndpointDeviceInformationSortOrder(pub i32);
				impl MidiEndpointDeviceInformationSortOrder {
					pub const None: Self = Self(0);
					pub const Name: Self = Self(1);
					pub const EndpointDeviceId: Self = Self(2);
					pub const DeviceInstanceId: Self = Self(3);
					pub const ContainerThenName: Self = Self(11);
					pub const ContainerThenEndpointDeviceId: Self = Self(12);
					pub const ContainerThenDeviceInstanceId: Self = Self(13);
					pub const TransportCodeThenName: Self = Self(21);
					pub const TransportCodeThenEndpointDeviceId: Self = Self(22);
					pub const TransportCodeThenDeviceInstanceId: Self = Self(23);
				}
				impl windows_core::imp::TypeKind for MidiEndpointDeviceInformationSortOrder {
					type TypeKind = windows_core::imp::CopyType;
				}
				impl windows_core::RuntimeType for MidiEndpointDeviceInformationSortOrder {
					const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"enum(Windows.Devices.Midi2.Enumeration.MidiEndpointDeviceInformationSortOrder;i4)",
					);
					const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
						b"Windows.Devices.Midi2.Enumeration.MidiEndpointDeviceInformationSortOrder",
					);
				}
			}
		}
	}
	pub mod Foundation {
		windows_core::imp::define_interface!(IClosable, IClosable_Vtbl, 0x30d5a829_7fa4_4026_83bb_d75bae4ea99e);
		impl windows_core::RuntimeType for IClosable {
			const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
			const NAME: windows_core::imp::ConstBuffer =
				windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.IClosable");
		}
		windows_core::imp::interface_hierarchy!(IClosable, windows_core::IUnknown, windows_core::IInspectable);
		impl IClosable {
			pub fn Close(&self) -> windows_core::Result<()> {
				unsafe { (windows_core::Interface::vtable(self).Close)(windows_core::Interface::as_raw(self)).ok() }
			}
		}
		impl windows_core::RuntimeName for IClosable {
			const NAME: &'static str = "Windows.Foundation.IClosable";
		}
		pub trait IClosable_Impl: windows_core::IUnknownImpl {
			fn Close(&self) -> windows_core::Result<()>;
		}
		impl IClosable_Vtbl {
			pub const fn new<Identity: IClosable_Impl, const OFFSET: isize>() -> Self {
				unsafe extern "system" fn Close<Identity: IClosable_Impl, const OFFSET: isize>(
					this: *mut core::ffi::c_void,
				) -> windows_core::HRESULT {
					unsafe {
						let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
						IClosable_Impl::Close(this).into()
					}
				}
				Self {
					base__: windows_core::IInspectable_Vtbl::new::<Identity, IClosable, OFFSET>(),
					Close: Close::<Identity, OFFSET>,
				}
			}
			pub fn matches(iid: &windows_core::GUID) -> bool {
				iid == &<IClosable as windows_core::Interface>::IID
			}
		}
		#[repr(C)]
		pub struct IClosable_Vtbl {
			pub base__: windows_core::IInspectable_Vtbl,
			pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
		}
		#[repr(transparent)]
		#[derive(Clone, Debug, Eq, PartialEq)]
		pub struct TypedEventHandler<TSender, TResult>(
			windows_core::IUnknown,
			core::marker::PhantomData<TSender>,
			core::marker::PhantomData<TResult>,
		)
		where
			TSender: windows_core::RuntimeType + 'static,
			TResult: windows_core::RuntimeType + 'static;
		unsafe impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
			windows_core::Interface for TypedEventHandler<TSender, TResult>
		{
			type Vtable = TypedEventHandler_Vtbl<TSender, TResult>;
			const IID: windows_core::GUID =
				windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
		}
		impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
			windows_core::RuntimeType for TypedEventHandler<TSender, TResult>
		{
			const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
				.push_slice(b"pinterface({9de1c534-6ae1-11e0-84e1-18a905bcc53f}")
				.push_slice(b";")
				.push_other(TSender::SIGNATURE)
				.push_slice(b";")
				.push_other(TResult::SIGNATURE)
				.push_slice(b")");
		}
		#[repr(C)]
		pub struct TypedEventHandler_Vtbl<TSender, TResult>
		where
			TSender: windows_core::RuntimeType + 'static,
			TResult: windows_core::RuntimeType + 'static,
		{
			base__: windows_core::IUnknown_Vtbl,
			Invoke: unsafe extern "system" fn(
				this: *mut core::ffi::c_void,
				sender: windows_core::imp::AbiType<TSender>,
				args: windows_core::imp::AbiType<TResult>,
			) -> windows_core::HRESULT,
			TSender: core::marker::PhantomData<TSender>,
			TResult: core::marker::PhantomData<TResult>,
		}
		struct TypedEventHandlerBox<
			TSender,
			TResult,
			F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()> + Send + 'static,
		>(core::marker::PhantomData<(TSender, TResult, fn() -> F)>)
		where
			TSender: windows_core::RuntimeType + 'static,
			TResult: windows_core::RuntimeType + 'static;
		impl<
			TSender: windows_core::RuntimeType + 'static,
			TResult: windows_core::RuntimeType + 'static,
			F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()> + Send + 'static,
		> TypedEventHandlerBox<TSender, TResult, F>
		{
			const VTABLE: TypedEventHandler_Vtbl<TSender, TResult> = TypedEventHandler_Vtbl::<TSender, TResult> {
				base__: windows_core::IUnknown_Vtbl {
					QueryInterface:
						windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::QueryInterface,
					AddRef: windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::AddRef,
					Release: windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::Release,
				},
				Invoke: Self::Invoke,
				TSender: core::marker::PhantomData::<TSender>,
				TResult: core::marker::PhantomData::<TResult>,
			};
			unsafe extern "system" fn Invoke(
				this: *mut core::ffi::c_void,
				sender: windows_core::imp::AbiType<TSender>,
				args: windows_core::imp::AbiType<TResult>,
			) -> windows_core::HRESULT {
				unsafe {
					let this = &mut *(this as *mut *mut core::ffi::c_void
						as *mut windows_core::imp::DelegateBox<TypedEventHandler<TSender, TResult>, F>);
					(this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&args)).into()
				}
			}
		}
	}
}
