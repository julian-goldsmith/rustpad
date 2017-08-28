//#![windows_subsystem = "windows"]

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

const IDA_ACCEL_TABLE: i32 = 10000;

// HACK: this will be available in the new version of winapi-rs
extern "system" {
    pub fn TranslateAcceleratorW(
        hWnd: HWND,
        hAccTable: HACCEL,
        lpMsg: LPMSG,
    ) -> c_int;
}

fn main() {
	let (main_window_ptr, main_window) = unsafe {
		let icc = INITCOMMONCONTROLSEX {
			dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
			dwICC: ICC_BAR_CLASSES | ICC_COOL_CLASSES,
		};
		
		comctl32::InitCommonControlsEx(&icc);
		
		let main_window_ptr = kernel32::LocalAlloc(0, mem::size_of::<MainWindow>() as u64) as *mut MainWindow;
		
		(main_window_ptr, main_window_ptr.as_mut().unwrap())
	};
	
	main_window.initialize();
	main_window.show();
	
	let accel = unsafe {
		LoadAcceleratorsW(main_window.instance, IDA_ACCEL_TABLE as WORD as ULONG_PTR as LPWSTR)
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
			if TranslateAcceleratorW(main_window.hwnd, accel, &mut msg) == 0 {
				TranslateMessage(&msg);
				DispatchMessageW(&msg);
			};
		};
	};
	
	unsafe {
		ptr::drop_in_place(main_window_ptr);
		kernel32::LocalFree(main_window_ptr as HLOCAL);
	};
}