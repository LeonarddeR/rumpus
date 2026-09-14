pub mod Windows {
	pub mod Win32 {
		#[inline]
		pub unsafe fn AllowSetForegroundWindow(dwprocessid: u32) -> windows_core::BOOL {
			windows_core::link!("user32.dll" "system" fn AllowSetForegroundWindow(dwprocessid : u32) -> windows_core::BOOL);
			unsafe { AllowSetForegroundWindow(dwprocessid) }
		}
		#[inline]
		pub unsafe fn CloseHandle(hobject: HANDLE) -> windows_core::BOOL {
			windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
			unsafe { CloseHandle(hobject) }
		}
		#[inline]
		pub unsafe fn ConnectNamedPipe(
			hnamedpipe: HANDLE,
			lpoverlapped: Option<*mut OVERLAPPED>,
		) -> windows_core::BOOL {
			windows_core::link!("kernel32.dll" "system" fn ConnectNamedPipe(hnamedpipe : HANDLE, lpoverlapped : *mut OVERLAPPED) -> windows_core::BOOL);
			unsafe { ConnectNamedPipe(hnamedpipe, lpoverlapped.unwrap_or(core::mem::zeroed()) as _) }
		}
		#[inline]
		pub unsafe fn CreateFileW<P0>(
			lpfilename: P0,
			dwdesiredaccess: u32,
			dwsharemode: u32,
			lpsecurityattributes: Option<*const SECURITY_ATTRIBUTES>,
			dwcreationdisposition: u32,
			dwflagsandattributes: u32,
			htemplatefile: Option<HANDLE>,
		) -> HANDLE
		where
			P0: windows_core::Param<windows_core::PCWSTR>,
		{
			windows_core::link!("kernel32.dll" "system" fn CreateFileW(lpfilename : windows_core::PCWSTR, dwdesiredaccess : u32, dwsharemode : u32, lpsecurityattributes : *const SECURITY_ATTRIBUTES, dwcreationdisposition : u32, dwflagsandattributes : u32, htemplatefile : HANDLE) -> HANDLE);
			unsafe {
				CreateFileW(
					lpfilename.param().abi(),
					dwdesiredaccess,
					dwsharemode,
					lpsecurityattributes.unwrap_or(core::mem::zeroed()) as _,
					dwcreationdisposition,
					dwflagsandattributes,
					htemplatefile.unwrap_or(core::mem::zeroed()) as _,
				)
			}
		}
		#[inline]
		pub unsafe fn CreateNamedPipeW<P0>(
			lpname: P0,
			dwopenmode: u32,
			dwpipemode: u32,
			nmaxinstances: u32,
			noutbuffersize: u32,
			ninbuffersize: u32,
			ndefaulttimeout: u32,
			lpsecurityattributes: Option<*const SECURITY_ATTRIBUTES>,
		) -> HANDLE
		where
			P0: windows_core::Param<windows_core::PCWSTR>,
		{
			windows_core::link!("kernel32.dll" "system" fn CreateNamedPipeW(lpname : windows_core::PCWSTR, dwopenmode : u32, dwpipemode : u32, nmaxinstances : u32, noutbuffersize : u32, ninbuffersize : u32, ndefaulttimeout : u32, lpsecurityattributes : *const SECURITY_ATTRIBUTES) -> HANDLE);
			unsafe {
				CreateNamedPipeW(
					lpname.param().abi(),
					dwopenmode,
					dwpipemode,
					nmaxinstances,
					noutbuffersize,
					ninbuffersize,
					ndefaulttimeout,
					lpsecurityattributes.unwrap_or(core::mem::zeroed()) as _,
				)
			}
		}
		#[inline]
		pub unsafe fn DisconnectNamedPipe(hnamedpipe: HANDLE) -> windows_core::BOOL {
			windows_core::link!("kernel32.dll" "system" fn DisconnectNamedPipe(hnamedpipe : HANDLE) -> windows_core::BOOL);
			unsafe { DisconnectNamedPipe(hnamedpipe) }
		}
		#[inline]
		pub unsafe fn ReadFile(
			hfile: HANDLE,
			lpbuffer: Option<*mut core::ffi::c_void>,
			nnumberofbytestoread: u32,
			lpnumberofbytesread: Option<*mut u32>,
			lpoverlapped: Option<*mut OVERLAPPED>,
		) -> windows_core::BOOL {
			windows_core::link!("kernel32.dll" "system" fn ReadFile(hfile : HANDLE, lpbuffer : *mut core::ffi::c_void, nnumberofbytestoread : u32, lpnumberofbytesread : *mut u32, lpoverlapped : *mut OVERLAPPED) -> windows_core::BOOL);
			unsafe {
				ReadFile(
					hfile,
					lpbuffer.unwrap_or(core::mem::zeroed()) as _,
					nnumberofbytestoread,
					lpnumberofbytesread.unwrap_or(core::mem::zeroed()) as _,
					lpoverlapped.unwrap_or(core::mem::zeroed()) as _,
				)
			}
		}
		#[inline]
		pub unsafe fn SetForegroundWindow(hwnd: HWND) -> windows_core::BOOL {
			windows_core::link!("user32.dll" "system" fn SetForegroundWindow(hwnd : HWND) -> windows_core::BOOL);
			unsafe { SetForegroundWindow(hwnd) }
		}
		#[inline]
		pub unsafe fn WaitNamedPipeW<P0>(lpnamedpipename: P0, ntimeout: u32) -> windows_core::BOOL
		where
			P0: windows_core::Param<windows_core::PCWSTR>,
		{
			windows_core::link!("kernel32.dll" "system" fn WaitNamedPipeW(lpnamedpipename : windows_core::PCWSTR, ntimeout : u32) -> windows_core::BOOL);
			unsafe { WaitNamedPipeW(lpnamedpipename.param().abi(), ntimeout) }
		}
		#[inline]
		pub unsafe fn WriteFile(
			hfile: HANDLE,
			lpbuffer: Option<*const core::ffi::c_void>,
			nnumberofbytestowrite: u32,
			lpnumberofbyteswritten: Option<*mut u32>,
			lpoverlapped: Option<*mut OVERLAPPED>,
		) -> windows_core::BOOL {
			windows_core::link!("kernel32.dll" "system" fn WriteFile(hfile : HANDLE, lpbuffer : *const core::ffi::c_void, nnumberofbytestowrite : u32, lpnumberofbyteswritten : *mut u32, lpoverlapped : *mut OVERLAPPED) -> windows_core::BOOL);
			unsafe {
				WriteFile(
					hfile,
					lpbuffer.unwrap_or(core::mem::zeroed()) as _,
					nnumberofbytestowrite,
					lpnumberofbyteswritten.unwrap_or(core::mem::zeroed()) as _,
					lpoverlapped.unwrap_or(core::mem::zeroed()) as _,
				)
			}
		}
		pub const ASFW_ANY: u32 = 4294967295;
		pub const ERROR_ACCESS_DENIED: i32 = 5;
		pub const ERROR_PIPE_CONNECTED: i32 = 535;
		pub const FILE_ATTRIBUTE_NORMAL: i32 = 128;
		pub const FILE_FLAG_FIRST_PIPE_INSTANCE: i32 = 524288;
		pub const GENERIC_WRITE: i32 = 1073741824;
		#[repr(transparent)]
		#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
		pub struct HANDLE(pub *mut core::ffi::c_void);
		#[repr(transparent)]
		#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
		pub struct HWND(pub *mut core::ffi::c_void);
		pub const INVALID_HANDLE_VALUE: HANDLE = HANDLE(-1 as _);
		pub const OPEN_EXISTING: i32 = 3;
		#[repr(C)]
		#[derive(Clone, Copy)]
		pub struct OVERLAPPED {
			pub Internal: usize,
			pub InternalHigh: usize,
			pub Anonymous: OVERLAPPED_0,
			pub hEvent: HANDLE,
		}
		impl Default for OVERLAPPED {
			fn default() -> Self {
				unsafe { core::mem::zeroed() }
			}
		}
		#[repr(C)]
		#[derive(Clone, Copy)]
		pub union OVERLAPPED_0 {
			pub Anonymous: OVERLAPPED_0_0,
			pub Pointer: *mut core::ffi::c_void,
		}
		impl Default for OVERLAPPED_0 {
			fn default() -> Self {
				unsafe { core::mem::zeroed() }
			}
		}
		#[repr(C)]
		#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
		pub struct OVERLAPPED_0_0 {
			pub Offset: u32,
			pub OffsetHigh: u32,
		}
		pub const PIPE_ACCESS_INBOUND: i32 = 1;
		pub const PIPE_READMODE_BYTE: i32 = 0;
		pub const PIPE_TYPE_BYTE: i32 = 0;
		pub const PIPE_UNLIMITED_INSTANCES: i32 = 255;
		pub const PIPE_WAIT: i32 = 0;
		#[repr(C)]
		#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
		pub struct SECURITY_ATTRIBUTES {
			pub nLength: u32,
			pub lpSecurityDescriptor: *mut core::ffi::c_void,
			pub bInheritHandle: windows_core::BOOL,
		}
	}
}
