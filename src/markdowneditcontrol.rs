//use comrak::{parse_document, ComrakOptions};
//use comrak::nodes::{AstNode, NodeValue};
use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
//use typed_arena::Arena;
use user32::*;
use winapi::*;
use gdi32::*;

pub const MARKDOWN_EDIT_CLASS: &str = "markdown_edit";

static mut richedit_wndproc: WNDPROC = None;
static mut richedit_wndextra: c_int = 0;

pub struct MarkdownEdit {
	pub text: String
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	match msg {
		_ => {
			CallWindowProcW(richedit_wndproc, hwnd, msg, w_param, l_param)
		},
	}
}

pub fn register() {
	let class: Vec<u16> = OsStr::new(MARKDOWN_EDIT_CLASS).encode_wide().chain(once(0)).collect();
	let richedit_class: Vec<u16> = OsStr::new("RichEdit20W").encode_wide().chain(once(0)).collect();
	
	unsafe {
		let mut wc: WNDCLASSW = mem::uninitialized();
		
		// FIXME: HINSTANCE
		GetClassInfoW(ptr::null_mut(), richedit_class.as_ptr(), &mut wc);
		
		richedit_wndproc = wc.lpfnWndProc;
		richedit_wndextra = wc.cbWndExtra;
		
		wc.style = wc.style & !CS_GLOBALCLASS;
		wc.lpfnWndProc = Some(wndproc);
		wc.lpszClassName = class.as_ptr();
		wc.cbWndExtra = richedit_wndextra + mem::size_of::<*mut MarkdownEdit>() as INT;
		
		RegisterClassW(&wc);
	};
}

pub fn unregister() {
	let class: Vec<u16> = OsStr::new(MARKDOWN_EDIT_CLASS).encode_wide().chain(once(0)).collect();
	
	unsafe {
		UnregisterClassW(class.as_ptr(), ptr::null_mut());
	};
}