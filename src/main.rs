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
use main_window::{MainWindow, MAIN_WINDOW_INSTANCE};

const IDA_ACCEL_TABLE: i32 = 10000;

// HACK: this will be in the new version of winapi-rs
extern "system" {
    pub fn TranslateAcceleratorW(
        hWnd: HWND,
        hAccTable: HACCEL,
        lpMsg: LPMSG,
    ) -> c_int;
}

fn main() {
	unsafe {
		let icc = INITCOMMONCONTROLSEX {
			dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
			dwICC: ICC_BAR_CLASSES | ICC_COOL_CLASSES,
		};
		
		comctl32::InitCommonControlsEx(&icc);
	};
	
	MainWindow::initialize();
	
	unsafe {	
		ShowWindow(MAIN_WINDOW_INSTANCE.hwnd, SW_SHOW);
		UpdateWindow(MAIN_WINDOW_INSTANCE.hwnd);
	};
	
	let accel = unsafe {
		LoadAcceleratorsW(MAIN_WINDOW_INSTANCE.hinstance, IDA_ACCEL_TABLE as WORD as ULONG_PTR as LPWSTR)
	};
	
	if accel == ptr::null_mut() {
		let error = unsafe {
			kernel32::GetLastError()
		};
		
		panic!("Couldn't create accel: {:?}", error);
	};
		
	unsafe {
		let mut msg: MSG = mem::uninitialized();
		
		while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
			if TranslateAcceleratorW(MAIN_WINDOW_INSTANCE.hwnd, accel, &mut msg) == 0 {
				TranslateMessage(&msg);
				DispatchMessageW(&msg);
			};
		};
	};
}