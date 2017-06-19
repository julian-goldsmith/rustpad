use comctl32;
use comrak;
use gdi32;
use kernel32;
use typed_arena;
use user32;
use winapi;

use comrak::{parse_document, ComrakOptions};
use comrak::nodes::{AstNode, NodeValue};
use std::ffi::OsStr;
use std::iter::{once,repeat};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use typed_arena::Arena;
use user32::*;
use winapi::*;
use gdi32::*;
use kernel32::LoadLibraryW;

pub const markdown_edit_class: &str = "MDEdit";

unsafe fn paint(hwnd: HWND) {
	let class: Vec<u16> = OsStr::new("Hello, world").encode_wide().chain(once(0)).collect();
	let mut ps: PAINTSTRUCT = mem::uninitialized();
	let mut ctl_rect: RECT = mem::uninitialized();
	let mut upd_rect: RECT = mem::uninitialized();
	
	GetClientRect(hwnd, &mut ctl_rect);
	GetUpdateRect(hwnd, &mut upd_rect, FALSE);
	
	let hdc = BeginPaint(hwnd, &mut ps);
	Rectangle(hdc, upd_rect.left, upd_rect.top, upd_rect.right, upd_rect.bottom);
	SetTextColor(hdc, RGB(0, 0, 0));
	SetBkMode(hdc, TRANSPARENT);
	DrawTextW(hdc, class.as_ptr(), -1, &mut ctl_rect, DT_SINGLELINE | DT_CENTER | DT_VCENTER);
	EndPaint(hwnd, &ps);
	
	ValidateRect(hwnd, &mut upd_rect);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	match msg {
		WM_PAINT => {
			paint(hwnd);
			0
		},
		_ => {
			DefWindowProcW(hwnd, msg, w_param, l_param)
		},
	}
}

pub fn register() {
	let class: Vec<u16> = OsStr::new(markdown_edit_class).encode_wide().chain(once(0)).collect();
	
	unsafe {
		let mut wc = WNDCLASSW {
			style: CS_GLOBALCLASS | CS_HREDRAW | CS_VREDRAW,
			lpfnWndProc: Some(wndproc),
			hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
			lpszClassName: class.as_ptr(),
			
			cbClsExtra: 0,
			cbWndExtra: 0,
			hInstance: ptr::null_mut(),
			hIcon: ptr::null_mut(),
			hbrBackground: 0 as HBRUSH,
			lpszMenuName: ptr::null_mut(),
		};
		
		RegisterClassW(&wc);
	};
}

pub fn unregister() {
	let class: Vec<u16> = OsStr::new(markdown_edit_class).encode_wide().chain(once(0)).collect();
	
	unsafe {
		UnregisterClassW(class.as_ptr(), ptr::null_mut());
	};
}