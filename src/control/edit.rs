use std::cmp;
use std::mem;
use std::panic;
use std::process;
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
	
	pub lines: Vec<Vec<u16>>,
	
	pub font: HFONT,
	pub font_height: i32,
	pub font_width: i32,
}

const EDIT_CLASS: &'static str = "R\0U\0S\0T\0P\0A\0D\0_\0E\0D\0I\0T\0\0\0";

impl Edit {
	unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		let result = panic::catch_unwind(|| {
			match msg { 
				WM_CLOSE => { user32::DestroyWindow(hwnd); },
				WM_NCCREATE => {
					let mut edit = (kernel32::LocalAlloc(0, mem::size_of::<Edit>() as u64) as *mut Edit).as_mut().unwrap();
					let lines = Vec::new();
					
					ptr::write(&mut edit.hwnd, hwnd);
					ptr::write(&mut edit.lines, lines);
					ptr::write(&mut edit.font, ptr::null_mut());
					edit.font_height = 0;
					edit.font_width = 0;
					
					edit.set_self();
					
					return TRUE as LRESULT; 
				},
				WM_DESTROY => {
					let edit_ptr = Edit::get_self(hwnd) as *mut Edit;
					ptr::drop_in_place(edit_ptr);
				},
				WM_NCDESTROY => {
					let edit_ptr = Edit::get_self(hwnd) as *mut Edit;
					kernel32::LocalFree(edit_ptr as HLOCAL);
				},
				WM_CREATE => {
					let mut edit = Edit::get_self(hwnd);
					println!("create {:?}", edit as *mut Edit as u64);
					
					let font = gdi32::GetStockObject(DEFAULT_GUI_FONT) as HFONT;
					edit.set_font(font);
					println!("create end");
				},
				WM_SIZE => {
					//let mut main_window = MainWindow::get_main_window(hwnd);
					
					//main_window.resize();
				},
				WM_COMMAND => {
					//let mut main_window = MainWindow::get_main_window(hwnd);
					
					//assert_eq!(hwnd, main_window.hwnd);
					
					//match LOWORD(w_param as u32) as i32 {
					//	ID_FILE_EXIT => { PostMessageW(hwnd, WM_CLOSE, 0, 0); },
					//	ID_FILE_NEW => main_window.clear_text(),
					//	ID_FILE_OPEN => main_window.open_file(),
					//	ID_FILE_SAVEAS => main_window.save_file(),
					//	_ => (),
					//};
				},
				WM_SETFONT => {
					let mut edit = Edit::get_self(hwnd);
					edit.set_font(w_param as HFONT);
				},
				WM_PAINT => {
					let mut edit = Edit::get_self(hwnd);
					
					println!("paint {:?}", edit as *mut Edit as u64);
					edit.on_paint();
					println!("paint end");
				},
				_ => { 
					return user32::DefWindowProcW(hwnd, msg, w_param, l_param); 
				},
			};
			
			0
		});
		
		// panicking doesn't work in functions called from C
		match result {
			Ok(val) => val,
			Err(err) => { 
				println!("Panicked: {:?}", err.downcast_ref::<String>());  
				process::exit(29);
			},
		}
	}
	
	unsafe fn get_self(hwnd: HWND) -> &'static mut Edit {
		(user32::GetWindowLongPtrW(hwnd, 0) as *mut Edit).as_mut().unwrap()
	}
	
	pub fn set_self(&mut self) {
		unsafe {
			kernel32::SetLastError(0);
			
			user32::SetWindowLongPtrW(self.hwnd, 0, self as *mut Edit as LONG_PTR);
			
			if kernel32::GetLastError() != 0 {
				panic!("Couldn't set window pointer: {}", kernel32::GetLastError());
			};
		};
	}
	
	pub fn register_class(instance: HINSTANCE) -> ATOM {
		unsafe {
			// FIXME: review this
			let wc = WNDCLASSEXW {
				cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
				style: 0,
				lpfnWndProc: Some(Edit::wndproc),
				cbClsExtra: 0,
				cbWndExtra: mem::size_of::<*mut Edit>() as i32,
				hInstance: instance,
				hIcon: user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION),
				hCursor: user32::LoadCursorW(ptr::null_mut(), IDC_ARROW),
				hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
				lpszMenuName: ptr::null_mut(),
				lpszClassName: EDIT_CLASS.as_ptr() as *const u16,
				hIconSm: user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION),
			};

			let class_atom = user32::RegisterClassExW(&wc);
			
			if class_atom == 0 {
				panic!("Couldn't register class: {:?}", kernel32::GetLastError());
			};
			
			class_atom
		}
	}
	
	pub fn set_font(&mut self, font: HFONT) {
		unsafe {
			let hdc = user32::GetDC(self.hwnd);
			let h_old = gdi32::SelectObject(hdc, font as HGDIOBJ);

			let mut tm: TEXTMETRICW = mem::zeroed();
			gdi32::GetTextMetricsW(hdc, &mut tm);

			// restore previous font
			gdi32::SelectObject(hdc, h_old);
			user32::ReleaseDC(self.hwnd, hdc);
			
			self.font = font;
			self.font_height = tm.tmHeight;
			self.font_width = tm.tmAveCharWidth;
		};
	}

	pub fn new(instance: HINSTANCE, parent: HWND, id: HMENU) -> &'static mut Edit {
		let blank: Vec<u16> = util::convert_string("");
		
		assert_ne!(parent, ptr::null_mut());
		
		let hwnd = unsafe {
			let hwnd = user32::CreateWindowExW(WS_EX_CLIENTEDGE, EDIT_CLASS.as_ptr() as *const u16, blank.as_ptr(),
				WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL,
				CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, 
				parent, id, instance, ptr::null_mut());
			
			if hwnd == ptr::null_mut() {
				panic!("Couldn't create edit {}", kernel32::GetLastError());
			};
			
			//user32::SendMessageW(edit.hwnd, WM_SETFONT, font as WPARAM, 0);
			//user32::SendMessageW(edit.hwnd, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
			
			hwnd
		};
		
		// the memory allocated is freed when WM_NCDESTROY is called
		unsafe {
			Edit::get_self(hwnd)
		}
	}
	
	unsafe fn on_paint(&mut self) {
		let mut ps: PAINTSTRUCT = mem::uninitialized();
		
		user32::BeginPaint(self.hwnd, &mut ps);
		
		let top_row = cmp::max(ps.rcPaint.top / self.font_height, 0) as usize;
		let bottom_row = cmp::min(ps.rcPaint.bottom / self.font_height, self.lines.len() as _) as usize;
		
		for i in top_row..bottom_row {
			self.paint_line(ps.hdc, i);
		}
		
		user32::EndPaint(self.hwnd, &ps);
	}
	
	unsafe fn paint_line(&mut self, hdc: HDC, line_no: usize) {
		let mut rect: RECT = mem::uninitialized();
		user32::GetClientRect(self.hwnd, &mut rect);
		
		rect.top = line_no as i32 * self.font_height;
		rect.bottom = (line_no as i32 + 1) * self.font_height;
		
		let line = &self.lines[line_no];
		
		user32::TabbedTextOutW(hdc, rect.left, rect.top, line.as_ptr(), line.len() as c_int, 4, ptr::null(), 0);
	}
	
	pub fn set_text(&mut self, text: &[u16]) {
		self.lines.clear();
	
		let mut line = Vec::<u16>::new();
		
		for c in text.iter() {
			if *c == '\n' as u16 {
				self.lines.push(line);
				line = Vec::new();
			} else if *c == '\r' as u16 {
				// skip carriage returns
			} else {
				line.push(*c);
			}
		}
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
	
	fn resize(&mut self) {
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