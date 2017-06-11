extern crate winapi;
extern crate user32;
extern crate kernel32;
use std::ffi::OsStr;
use std::ffi::CString;
use std::io::Error;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::mem;
use user32::*;
use winapi::*;
use winapi::winuser::*;

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	match msg {
		WM_CLOSE => { DestroyWindow(hwnd); },
		WM_DESTROY => { PostQuitMessage(0); },
		_ => { return DefWindowProcW(hwnd, msg, w_param, l_param); },
	};
	0
}

fn main() {
	let class_name: Vec<u16> = OsStr::new("myWindowClass").encode_wide().chain(once(0)).collect();
	let window_title: Vec<u16> = OsStr::new("Test window").encode_wide().chain(once(0)).collect();
	
	unsafe {
		let h_instance = kernel32::GetModuleHandleW(ptr::null_mut());
	
		let wc = WNDCLASSEXW {
			cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
			style: 0,
			lpfnWndProc: Some(wndproc),
			cbClsExtra: 0,
			cbWndExtra: 0,
			hInstance: h_instance,
			hIcon: LoadIconW(ptr::null_mut(), IDI_APPLICATION),
			hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
			hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
			lpszMenuName: ptr::null_mut(),
			lpszClassName: class_name.as_ptr(),
			hIconSm: LoadIconW(ptr::null_mut(), IDI_APPLICATION),
		};
		
		if RegisterClassExW(&wc) == 0 {
			println!("Couldn't register class");
			return;
		};
		
		let hwnd = CreateWindowExW(
			WS_EX_CLIENTEDGE, class_name.as_ptr(), window_title.as_ptr(),
			WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT, 240, 120,
			ptr::null_mut(), ptr::null_mut(), h_instance, ptr::null_mut());
			
		if hwnd == ptr::null_mut() {
			println!("Couldn't create window");
			return;
		};
		
		ShowWindow(hwnd, SW_SHOW);
		UpdateWindow(hwnd);
		
		let mut msg: MSG = mem::uninitialized();
		
		while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
			TranslateMessage(&msg);
			DispatchMessageW(&msg);
		};
	};
}