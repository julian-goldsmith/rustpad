use std::mem;
use std::ptr;
use winapi::*;
use user32;
use kernel32;
use util;
use control::Control;

#[derive(Debug)]
pub struct Toolbar {
	pub hwnd: HWND,
}

const ID_MENU: HMENU = 9000 as HMENU;
const ID_FILE_EXIT: i32 = 9001;
const ID_FILE_NEW: i32 = 9002;
const ID_FILE_OPEN: i32 = 9003;
const ID_FILE_SAVEAS: i32 = 9004;

impl Toolbar {
	pub fn new(instance: HINSTANCE, parent: HWND, id: HMENU) -> Toolbar {
		let toolbar_name = util::convert_string("ToolbarWindow32");
		
		assert_ne!(parent, ptr::null_mut());
			
		let tbab = TBADDBITMAP {
			hInst: -1 as i64 as HINSTANCE,//HINST_COMMCTRL,
			nID: IDB_STD_SMALL_COLOR,
		};
		
		let tbb: [TBBUTTON; 3] = [
			TBBUTTON {
				iBitmap: STD_FILENEW,
				idCommand: ID_FILE_NEW,
				fsState: TBSTATE_ENABLED,
				fsStyle: TBSTYLE_BUTTON as u8,
				bReserved: [0; 6],
				dwData: 0,
				iString: 0,
			},
			TBBUTTON {
				iBitmap: STD_FILEOPEN,
				idCommand: ID_FILE_OPEN,
				fsState: TBSTATE_ENABLED,
				fsStyle: TBSTYLE_BUTTON as u8,
				bReserved: [0; 6],
				dwData: 0,
				iString: 0,
			},
			TBBUTTON {
				iBitmap: STD_FILESAVE,
				idCommand: ID_FILE_SAVEAS,
				fsState: TBSTATE_ENABLED,
				fsStyle: TBSTYLE_BUTTON as u8,
				bReserved: [0; 6],
				dwData: 0,
				iString: 0,
			},
		];
		
		let mut tool = Toolbar {
			hwnd: unsafe {
				user32::CreateWindowExW(0, toolbar_name.as_ptr(), ptr::null_mut(), 
					WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, 
					parent, id, instance, ptr::null_mut())
			},
		};
		
		if tool.hwnd == ptr::null_mut() {
			panic!("Couldn't create toolbar");
		};
		
		tool.add_bitmap(&tbab);
		tool.add_buttons(&tbb);
		
		tool
	}
	
	pub fn add_bitmap(&mut self, bitmap: &TBADDBITMAP) {
		unsafe {
			user32::SendMessageW(self.hwnd, TB_ADDBITMAP, 0, bitmap as *const TBADDBITMAP as LPARAM);
		};
	}
	
	pub fn add_buttons(&mut self, buttons: &[commctrl::TBBUTTON]) {
		unsafe {
			user32::SendMessageW(self.hwnd, TB_BUTTONSTRUCTSIZE, mem::size_of::<commctrl::TBBUTTON>() as WPARAM, 0);		
			user32::SendMessageW(self.hwnd, TB_ADDBUTTONSW, buttons.len() as u64, buttons.as_ptr() as LPARAM);
		};
	}
}

impl Control for Toolbar {
	fn get_hwnd(&self) -> HWND {
		self.hwnd
	}
	
	fn resize(&mut self) {
		unsafe {
			user32::SendMessageW(self.hwnd, WM_SIZE, 0, 0);
		}
	}
	
	fn get_size(&self) -> RECT {
		unsafe {
			let mut rect: RECT = mem::uninitialized();
			user32::GetWindowRect(self.hwnd, &mut rect);
			rect
		}
	}
}