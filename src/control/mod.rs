use winapi::*;

mod edit;
mod statusbar;

pub use self::edit::Edit;
pub use self::statusbar::Statusbar;

pub trait Control {
	fn get_hwnd(&self) -> HWND;
	fn resize(&mut self);
	fn get_size(&self) -> RECT;
}
