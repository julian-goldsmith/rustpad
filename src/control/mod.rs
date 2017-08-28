use winapi::*;

mod edit;

pub use self::edit::Edit;

trait Control {
	fn get_hwnd(&self) -> HWND;
	fn size(&mut self);
	fn get_size(&self) -> RECT;
}
