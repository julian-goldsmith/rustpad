extern crate comctl32;
extern crate comdlg32;
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate winapi;

mod main_window;

use std::mem;
use std::ptr;
use user32::*;
use winapi::*;
use main_window::MainWindow;

fn main() {
	unsafe {
		let icc = INITCOMMONCONTROLSEX {
			dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
			dwICC: ICC_BAR_CLASSES | ICC_COOL_CLASSES,
		};
		
		comctl32::InitCommonControlsEx(&icc);
	};
	
	let main_window = MainWindow::get_instance();
	
	unsafe {	
		ShowWindow(main_window.hwnd, SW_SHOW);
		UpdateWindow(main_window.hwnd);
	};
	
	unsafe {
		let mut msg: MSG = mem::uninitialized();
		
		while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
			TranslateMessage(&msg);
			DispatchMessageW(&msg);
		};
	};
}