use std::mem;
use std::ptr;
use winapi::*;
use user32;
use kernel32;
use util;
use control::Control;

#[derive(Debug)]
pub struct Statusbar {
	pub hwnd: HWND,
}

impl Statusbar {
	pub fn new(instance: HINSTANCE, parent: HWND, id: HMENU) -> Statusbar {
		let statusbar_class = util::convert_string("msctls_statusbar32");
		
		assert_ne!(parent, ptr::null_mut());
		
		let mut status = Statusbar {
			hwnd: ptr::null_mut(),
		};
		
		unsafe {
			status.hwnd = 
				user32::CreateWindowExW(0, statusbar_class.as_ptr(), ptr::null_mut(),
					WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP, 0, 0, 0, 0,
					parent, id, instance, ptr::null_mut());
			
			if status.hwnd == ptr::null_mut() {
				panic!("Couldn't create statusbar {}", kernel32::GetLastError());
			};
		};
		
		status
	}
	
	pub fn set_text(&mut self, text: &[u16]) {
		unsafe {
			user32::SendMessageW(self.hwnd, WM_SETTEXT, 0, text.as_ptr() as LPARAM);
		};
	}
}

impl Control for Statusbar {
	fn get_hwnd(&self) -> HWND {
		self.hwnd
	}
	
	fn resize(&mut self) {
		unsafe {
			user32::SendMessageW(self.hwnd, WM_SIZE, 0, 0);
		}
	}
	
	fn get_size(&self) -> RECT {
		unsafe {
			let mut rect: RECT = mem::uninitialized();
			user32::GetWindowRect(self.hwnd, &mut rect);
			rect
		}
	}
}