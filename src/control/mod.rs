use winapi::*;

mod edit;
mod statusbar;
mod toolbar;

pub use self::edit::Edit;
pub use self::statusbar::Statusbar;
pub use self::toolbar::Toolbar;

pub trait Control {
	fn get_hwnd(&self) -> HWND;
	fn resize(&mut self);
	fn get_size(&self) -> RECT;
}
