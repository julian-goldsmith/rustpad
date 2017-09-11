use std::mem;
use winapi::*;
use comdlg32::*;
use kernel32;
use util;
use std::iter::{once, Iterator};

pub unsafe fn open_file_dialog(parent: HWND) -> Option<Vec<wchar_t>> {
	let mut ofn: OPENFILENAMEW = mem::zeroed();
	let mut filename_buf = [0 as wchar_t; 1024];
	let filter_text = util::convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");	// TODO: pass filters in
	let default_ext = util::convert_string("txt");
	
	ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
	ofn.hwndOwner = parent;
	ofn.lpstrFilter = filter_text.as_ptr();
	ofn.lpstrFile = filename_buf.as_mut_ptr();
	ofn.nMaxFile = filename_buf.len() as u32;
	ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;
	ofn.lpstrDefExt = default_ext.as_ptr();
	
	match GetOpenFileNameW(&mut ofn) {
		FALSE => None,
		_ => {
			let filename_length = kernel32::lstrlenW(ofn.lpstrFile) as usize;
			let filename = (&filename_buf[0..filename_length + 1]).to_vec();
			Some(filename)
		},
	}
}

pub unsafe fn save_file_dialog(parent: HWND) -> Option<Vec<wchar_t>> {
	let mut ofn: OPENFILENAMEW = mem::zeroed();
	let mut filename_buf = [0 as wchar_t; 1024];
	let filter_text = util::convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
	let default_ext = util::convert_string("txt");
	
	ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
	ofn.hwndOwner = parent;
	ofn.lpstrFilter = filter_text.as_ptr();
	ofn.lpstrFile = filename_buf.as_mut_ptr();
	ofn.nMaxFile = filename_buf.len() as u32;
	ofn.Flags = OFN_EXPLORER | OFN_HIDEREADONLY;
	ofn.lpstrDefExt = default_ext.as_ptr();
	
	match GetSaveFileNameW(&mut ofn) {
		FALSE => None,
		_ => {
			let filename_length = kernel32::lstrlenW(ofn.lpstrFile) as usize;
			let filename = (&filename_buf[0..filename_length]).iter().map(|c| *c).chain(once(0)).collect();
			Some(filename)
		},
	}
}