use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

pub fn convert_string(string: &str) -> Vec<u16> {
	OsStr::new(string).encode_wide().chain(once(0)).collect()
}