use std::mem;
use std::ptr;
use winapi::*;
use user32;
use gdi32;
use kernel32;
use util;
use control::Control;

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
	
	pub fn set_text(&mut self, text: &[u16]) {
		unsafe {
			user32::SendMessageW(self.hwnd, WM_SETTEXT, 0, text.as_ptr() as LPARAM);
		};
	}
	
	pub fn get_text(&self) -> Vec<u16> {
		unsafe {
			let text_buf = [0 as u16; 4096];
			user32::SendMessageW(self.hwnd, WM_GETTEXT, text_buf.len() as u64, text_buf.as_ptr() as LPARAM);
			
			let text_length = kernel32::lstrlenW(text_buf.as_ptr()) as usize;
			
			text_buf[0..text_length].to_vec()
		}
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