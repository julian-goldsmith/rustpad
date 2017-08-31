use std::mem;
use std::ptr;
use winapi::*;
use user32;
use util;
use control::Control;

#[derive(Debug)]
pub struct Toolbar {
	pub hwnd: HWND,
}

impl Toolbar {
	pub fn new(instance: HINSTANCE, parent: HWND, id: HMENU) -> Toolbar {
		assert_ne!(parent, ptr::null_mut());
		
		let toolbar_name = util::convert_string("ToolbarWindow32");
		
		let tool = Toolbar {
			hwnd: unsafe {
				user32::CreateWindowExW(0, toolbar_name.as_ptr(), ptr::null_mut(), 
					WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, 
					parent, id, instance, ptr::null_mut())
			},
		};
		
		if tool.hwnd == ptr::null_mut() {
			panic!("Couldn't create toolbar");
		};
		
		tool
	}
	
	pub fn add_bitmap(&mut self, bitmap: &TBADDBITMAP) {
		unsafe {
			user32::SendMessageW(self.hwnd, TB_ADDBITMAP, 0, bitmap as *const TBADDBITMAP as LPARAM);
		};
	}
	
	pub fn add_buttons(&mut self, buttons: &[commctrl::TBBUTTON]) {
		unsafe {
			user32::SendMessageW(self.hwnd, TB_BUTTONSTRUCTSIZE, mem::size_of::<commctrl::TBBUTTON>() as WPARAM, 0);		
			user32::SendMessageW(self.hwnd, TB_ADDBUTTONSW, buttons.len() as u64, buttons.as_ptr() as LPARAM);
		};
	}
}

impl Control for Toolbar {
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