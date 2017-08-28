use std::mem;
use std::ptr;
use winapi::*;
use user32;
use gdi32;
use kernel32;
use util;

trait Control {
	fn get_hwnd(&self) -> HWND;
	fn size(&mut self);
	fn get_size(&self) -> RECT;
}

#[derive(Debug)]
pub struct Edit {
	pub hwnd: HWND,
}

impl Edit {
	pub fn new(instance: HINSTANCE, parent: HWND, id: HMENU) -> Edit {
		let edit_class: Vec<u16> = util::convert_string("EDIT");
		let blank: Vec<u16> = util::convert_string("");
		
		assert_ne!(parent, ptr::null_mut());
		
		let mut edit = Edit {
			hwnd: ptr::null_mut(),
		};
		
		unsafe {
			edit.hwnd = 
				user32::CreateWindowExW(WS_EX_CLIENTEDGE, edit_class.as_ptr(), blank.as_ptr(),
					WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL,
					CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, 
					parent, id, instance, ptr::null_mut());
			
			if edit.hwnd == ptr::null_mut() {
				panic!("Couldn't create edit {}", kernel32::GetLastError());
			};
			
			let font = gdi32::GetStockObject(DEFAULT_GUI_FONT) as HFONT;
			user32::SendMessageW(edit.hwnd, WM_SETFONT, font as WPARAM, 0);
			
			user32::SendMessageW(edit.hwnd, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
		};
		
		edit
	}
}

impl Control for Edit {
	fn get_hwnd(&self) -> HWND {
		self.hwnd
	}
	
	fn size(&mut self) {
		// TODO
	}
	
	fn get_size(&self) -> RECT {
		unsafe {
			let mut rect: RECT = mem::uninitialized();
			user32::GetWindowRect(self.hwnd, &mut rect);
			rect
		}
	}
}