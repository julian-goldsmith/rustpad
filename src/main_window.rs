use std::ffi::{OsStr,OsString};
use std::iter::once;
use std::mem;
use std::os::windows::ffi::{OsStrExt,OsStringExt};
use std::ptr;
use std::io::{Read,Write};
use user32::*;
use winapi::*;
use gdi32::*;
use comdlg32::*;
use kernel32;
use std::fs::File;
use std::path::Path;
use std::str;

const IDC_EDIT: i32 = 101;
const IDC_TOOLBAR: i32 = 102;
const IDC_MAIN_STATUS: i32 = 104;

const ID_MENU: HMENU = 9000 as HMENU;
const ID_FILE_EXIT: i32 = 9001;
const ID_FILE_NEW: i32 = 9002;
const ID_FILE_OPEN: i32 = 9003;
const ID_FILE_SAVEAS: i32 = 9004;

fn convert_string(string: &str) -> Vec<u16> {
	OsStr::new(string).encode_wide().chain(once(0)).collect()
}

pub struct MainWindow {
	pub hwnd: HWND,
	pub edit: HWND,
	pub toolbar: HWND,
	pub status: HWND,
	pub hinstance: HINSTANCE,
}

pub static mut MAIN_WINDOW_INSTANCE: MainWindow = MainWindow {
	hwnd: 0 as HWND,
	edit: 0 as HWND,
	toolbar: 0 as HWND,
	status: 0 as HWND,
	hinstance: 0 as HINSTANCE,
};

impl MainWindow {
	unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		match msg { 
			WM_CLOSE => { DestroyWindow(hwnd); },
			WM_DESTROY => { PostQuitMessage(0); },
			WM_CREATE => { 
				let create = l_param as LPCREATESTRUCTA;
				
				MAIN_WINDOW_INSTANCE.hwnd = hwnd;
				MAIN_WINDOW_INSTANCE.hinstance = (*create).hInstance;
			
				MAIN_WINDOW_INSTANCE.populate_window();
			},
			WM_SIZE => {
				MAIN_WINDOW_INSTANCE.resize();
			},
			WM_COMMAND => {
				match LOWORD(w_param as u32) as i32 {
					ID_FILE_EXIT => { PostMessageW(hwnd, WM_CLOSE, 0, 0); },
					ID_FILE_NEW => MAIN_WINDOW_INSTANCE.clear_text(),
					ID_FILE_OPEN => MAIN_WINDOW_INSTANCE.open_file(),
					ID_FILE_SAVEAS => MAIN_WINDOW_INSTANCE.save_file(),
					_ => (),
				};
			},
			_ => { 
				return DefWindowProcW(hwnd, msg, w_param, l_param); 
			},
		};
		0
	}

	unsafe fn open_file(&mut self) {
		let mut ofn: OPENFILENAMEW = mem::zeroed();
		let mut filename_buf = [0 as u16; 1024];
		let filter_text = convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
		let default_ext = convert_string("txt");
		
		ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
		ofn.hwndOwner = self.hwnd;
		ofn.lpstrFilter = filter_text.as_ptr();
		ofn.lpstrFile = filename_buf.as_mut_ptr();
		ofn.nMaxFile = filename_buf.len() as u32;
		ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;
		ofn.lpstrDefExt = default_ext.as_ptr();
		
		if GetOpenFileNameW(&mut ofn) != FALSE {
			// load edit
			let filename_length = kernel32::lstrlenW(ofn.lpstrFile) as usize;
			let filename = OsString::from_wide(&filename_buf[0..filename_length]).to_string_lossy().into_owned();
			let mut file = File::open(&Path::new(&filename)).unwrap();
			
			let mut data = String::new();
			
			file.read_to_string(&mut data).expect("Read file");
			
			let text = convert_string(&data);
			SendMessageW(self.edit, WM_SETTEXT, 0, text.as_ptr() as LPARAM);
		}
	}

	unsafe fn save_file(&mut self) {
		let mut ofn: OPENFILENAMEW = mem::zeroed();
		let mut filename_buf = [0 as u16; 1024];
		let filter_text = convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
		let default_ext = convert_string("txt");
		
		ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
		ofn.hwndOwner = self.hwnd;
		ofn.lpstrFilter = filter_text.as_ptr();
		ofn.lpstrFile = filename_buf.as_mut_ptr();
		ofn.nMaxFile = filename_buf.len() as u32;
		ofn.Flags = OFN_EXPLORER | OFN_HIDEREADONLY;
		ofn.lpstrDefExt = default_ext.as_ptr();
		
		if GetSaveFileNameW(&mut ofn) != FALSE {
			// load edit
			let filename_length = kernel32::lstrlenW(ofn.lpstrFile) as usize;
			let filename = OsString::from_wide(&filename_buf[0..filename_length]).to_string_lossy().into_owned();
			let mut file = File::create(&Path::new(&filename)).unwrap();
			
			let text_buf = [0 as u16; 4096];
			SendMessageW(self.edit, WM_GETTEXT, text_buf.len() as u64, text_buf.as_ptr() as LPARAM);
			
			let text_length = kernel32::lstrlenW(ofn.lpstrFile) as usize;
			let file_text = OsString::from_wide(&text_buf[0..text_length]).to_string_lossy().into_owned();
			
			file.write_all(&file_text.as_bytes()).expect("Write file");
		}
	}

	unsafe fn clear_text(&mut self) {
		let blank: Vec<u16> = convert_string("");
		
		SendMessageW(self.edit, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
	}

	unsafe fn resize(&mut self) {
		let mut rect: RECT = mem::uninitialized();
		GetClientRect(self.hwnd, &mut rect);
		
		let mut tool_rect: RECT = mem::uninitialized();
		SendMessageW(self.toolbar, WM_SIZE, 0, 0);
		GetWindowRect(self.toolbar, &mut tool_rect);
		
		let mut status_rect: RECT = mem::uninitialized();
		SendMessageW(self.status, WM_SIZE, 0, 0);
		GetWindowRect(self.status, &mut status_rect);
		
		let tool_height = tool_rect.bottom - tool_rect.top;
		let status_height = status_rect.bottom - status_rect.top;
		let edit_height = rect.bottom - tool_height - status_height;
		
		SetWindowPos(self.edit, ptr::null_mut(), 0, tool_height, rect.right, edit_height, SWP_NOZORDER);
	}

	pub fn register_window_class(h_instance: HINSTANCE, class_name: &Vec<u16>) -> ATOM {
		let atom = unsafe {
			let wc = WNDCLASSEXW {
				cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
				style: 0,
				lpfnWndProc: Some(MainWindow::wndproc),
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

			RegisterClassExW(&wc)
		};
		
		if atom == 0 {
			panic!("Couldn't register class");
		};
		
		atom
	}

	pub fn create_window(instance: HINSTANCE, class_name: &Vec<u16>, window_title: &Vec<u16>) {
		let hmenu = unsafe {
			LoadMenuW(instance, ID_MENU as WORD as ULONG_PTR as LPWSTR)
		};
		
		if hmenu == ptr::null_mut() {
			let error = unsafe {
				kernel32::GetLastError()
			};
			
			panic!("Couldn't create hmenu: {:?}", error);
		};
		
		let hwnd = unsafe {
			CreateWindowExW(
				0, class_name.as_ptr(), window_title.as_ptr(),
				WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 480, 320,
				ptr::null_mut(), hmenu, instance, ptr::null_mut())
		};
		
		if hwnd == ptr::null_mut() {
			let error = unsafe {
				kernel32::GetLastError()
			};
			
			panic!("Couldn't create window: {:?}", error);
		};
	}

	fn create_edit(&mut self) {
		let edit_class: Vec<u16> = convert_string("EDIT");
		let blank: Vec<u16> = convert_string("");
		
		self.edit = unsafe {
			CreateWindowExW(WS_EX_CLIENTEDGE, edit_class.as_ptr(), blank.as_ptr(),
				WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL,
				CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, 
				self.hwnd, IDC_EDIT as HMENU, self.hinstance, ptr::null_mut())
		};
		
		if self.edit == ptr::null_mut() {
			panic!("Couldn't create edit");
		};
		
		unsafe {
			let font = GetStockObject(DEFAULT_GUI_FONT) as HFONT;
			SendMessageW(self.edit, WM_SETFONT, font as WPARAM, 0);
		
			SendMessageW(self.edit, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
		};
	}

	fn create_toolbar(&mut self) {
		let toolbar_name = convert_string("ToolbarWindow32");
		
		self.toolbar = unsafe {
			CreateWindowExW(0, toolbar_name.as_ptr(), ptr::null_mut(), WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, 
				self.hwnd, IDC_TOOLBAR as HMENU, self.hinstance, ptr::null_mut())
		};
		
		if self.toolbar == ptr::null_mut() {
			panic!("Couldn't create toolbar");
		};
		
		unsafe {
			SendMessageW(self.toolbar, TB_BUTTONSTRUCTSIZE, mem::size_of::<commctrl::TBBUTTON>() as WPARAM, 0);
		};
		
		let tbab = TBADDBITMAP {
			hInst: -1 as i64 as HINSTANCE,//HINST_COMMCTRL,
			nID: IDB_STD_SMALL_COLOR,
		};
		
		unsafe {
			SendMessageW(self.toolbar, TB_ADDBITMAP, 0, &tbab as *const TBADDBITMAP as LPARAM)
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
			
		unsafe {
			SendMessageW(self.toolbar, TB_ADDBUTTONSW, tbb.len() as u64, tbb.as_ptr() as LPARAM);
		};
	}

	fn create_status(&mut self) {
		let statusbar_class = convert_string("msctls_statusbar32");
		
		unsafe {
			MAIN_WINDOW_INSTANCE.status =
				CreateWindowExW(0, statusbar_class.as_ptr(), ptr::null_mut(),
					WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP, 0, 0, 0, 0,
					self.hwnd, IDC_MAIN_STATUS as HMENU, self.hinstance, ptr::null_mut());
		
			if MAIN_WINDOW_INSTANCE.status == ptr::null_mut() {
				panic!("Couldn't create status");
			};
		};
	}

	fn populate_window(&mut self) {
		self.create_edit();
		self.create_toolbar();
		self.create_status();
	}

	fn get_current_instance_handle() -> HINSTANCE {
		unsafe {
			kernel32::GetModuleHandleW(ptr::null_mut())
		}
	}
	
	pub fn initialize() {
		unsafe {
			assert_eq!(MAIN_WINDOW_INSTANCE.hwnd, ptr::null_mut());
		};
	
		let class_name: Vec<u16> = convert_string("Rustpad");
		let window_title: Vec<u16> = convert_string("Rustpad");
	
		let instance = MainWindow::get_current_instance_handle();
	
		MainWindow::register_window_class(instance, &class_name);
		
		MainWindow::create_window(instance, &class_name, &window_title);
	}
}