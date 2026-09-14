pub mod Windows {
	pub mod Win32 {
		#[inline]
		pub unsafe fn MapVirtualKeyW(ucode: u32, umaptype: u32) -> u32 {
			windows_core::link!("user32.dll" "system" fn MapVirtualKeyW(ucode : u32, umaptype : u32) -> u32);
			unsafe { MapVirtualKeyW(ucode, umaptype) }
		}
		#[inline]
		pub unsafe fn SendInput(pinputs: &[INPUT], cbsize: i32) -> u32 {
			windows_core::link!("user32.dll" "system" fn SendInput(cinputs : u32, pinputs : *const INPUT, cbsize : i32) -> u32);
			unsafe { SendInput(pinputs.len().try_into().unwrap(), pinputs.as_ptr(), cbsize) }
		}
		#[repr(C)]
		#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
		pub struct HARDWAREINPUT {
			pub uMsg: u32,
			pub wParamL: u16,
			pub wParamH: u16,
		}
		#[repr(C)]
		#[derive(Clone, Copy)]
		pub struct INPUT {
			pub r#type: u32,
			pub Anonymous: INPUT_0,
		}
		impl Default for INPUT {
			fn default() -> Self {
				unsafe { core::mem::zeroed() }
			}
		}
		#[repr(C)]
		#[derive(Clone, Copy)]
		pub union INPUT_0 {
			pub mi: MOUSEINPUT,
			pub ki: KEYBDINPUT,
			pub hi: HARDWAREINPUT,
		}
		impl Default for INPUT_0 {
			fn default() -> Self {
				unsafe { core::mem::zeroed() }
			}
		}
		pub const INPUT_KEYBOARD: i32 = 1;
		#[repr(C)]
		#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
		pub struct KEYBDINPUT {
			pub wVk: u16,
			pub wScan: u16,
			pub dwFlags: u32,
			pub time: u32,
			pub dwExtraInfo: usize,
		}
		pub const KEYEVENTF_EXTENDEDKEY: i32 = 1;
		pub const KEYEVENTF_KEYUP: i32 = 2;
		pub const MAPVK_VK_TO_VSC: i32 = 0;
		#[repr(C)]
		#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
		pub struct MOUSEINPUT {
			pub dx: i32,
			pub dy: i32,
			pub mouseData: u32,
			pub dwFlags: u32,
			pub time: u32,
			pub dwExtraInfo: usize,
		}
		pub const VK_CONTROL: i32 = 17;
		pub const VK_DOWN: i32 = 40;
		pub const VK_RETURN: i32 = 13;
		pub const VK_SHIFT: i32 = 16;
		pub const VK_UP: i32 = 38;
	}
}
