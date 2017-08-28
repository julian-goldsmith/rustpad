use std::ffi::{OsStr,OsString};
use std::iter::once;
use std::mem;
use std::os::windows::ffi::{OsStrExt,OsStringExt};
use std::ptr;
use std::io::{Read,Write};
use std::panic;
use user32::*;
use winapi::*;
use gdi32::*;
use comdlg32::*;
use kernel32;
use std::fs::File;
use std::path::Path;
use std::str;
use std::process;

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
	pub instance: HINSTANCE,
	pub class_atom: ATOM,
}

impl MainWindow {
	unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		println!("wndproc {:?} {:?}", hwnd, msg);
	
		let result = panic::catch_unwind(|| {
			match msg { 
				WM_CLOSE => { DestroyWindow(hwnd); },
				WM_DESTROY => { PostQuitMessage(0); },
				WM_NCCREATE => {
					let cs = l_param as *mut CREATESTRUCTW;
					let mut main_window = ((*cs).lpCreateParams as *mut MainWindow).as_mut().unwrap();
					
					main_window.hwnd = hwnd;
					
					SetWindowLongPtrW(hwnd, 0, main_window as *mut MainWindow as LONG_PTR);
					println!("hwnd {:?} {:?}", hwnd, main_window.hwnd);
					
					return TRUE as LRESULT; 
				},
				WM_CREATE => {
					let mut main_window = (GetWindowLongPtrW(hwnd, 0) as *mut MainWindow).as_mut().unwrap();
					println!("hwnd {:?} {:?}", hwnd, main_window.hwnd);
					
					main_window.populate_window();
				},
				WM_SIZE => {
					let mut main_window = (GetWindowLongPtrW(hwnd, 0) as *mut MainWindow).as_mut().unwrap();
					println!("hwnd {:?} {:?}", hwnd, main_window.hwnd);
					
					loop {}; 
					
					assert_eq!(main_window.hwnd, hwnd);
					
					main_window.resize();
				},
				WM_COMMAND => {
					let mut main_window = (GetWindowLongPtrW(hwnd, 0) as *mut MainWindow).as_mut().unwrap();
					
					match LOWORD(w_param as u32) as i32 {
						ID_FILE_EXIT => { PostMessageW(hwnd, WM_CLOSE, 0, 0); },
						ID_FILE_NEW => main_window.clear_text(),
						ID_FILE_OPEN => main_window.open_file(),
						ID_FILE_SAVEAS => main_window.save_file(),
						_ => (),
					};
				},
				_ => { 
					return DefWindowProcW(hwnd, msg, w_param, l_param); 
				},
			};
			
			0
		});
		
		// panicking doesn't work in functions called from C
		match result {
			Ok(val) => val,
			Err(err) => { 
				println!("Panicked: {:?}", err.downcast_ref::<String>());  
				process::exit(29); 
			},
		}
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
		
		println!("Edit height {} {} {} {}", edit_height, tool_height, status_height, rect.bottom);
		
		SetWindowPos(self.edit, ptr::null_mut(), 0, tool_height, rect.right, edit_height, SWP_NOZORDER);
	}
	
	pub fn show(&self) {
		unsafe {
			ShowWindow(self.hwnd, SW_SHOW);
			UpdateWindow(self.hwnd);
		};
	}

	pub fn register_window_class(&mut self, class_name: &Vec<u16>) {
		self.class_atom = unsafe {
			let wc = WNDCLASSEXW {
				cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
				style: 0,
				lpfnWndProc: Some(MainWindow::wndproc),
				cbClsExtra: 0,
				cbWndExtra: mem::size_of::<*mut MainWindow>() as i32,
				hInstance: self.instance,
				hIcon: LoadIconW(ptr::null_mut(), IDI_APPLICATION),
				hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
				hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
				lpszMenuName: ptr::null_mut(),
				lpszClassName: class_name.as_ptr(),
				hIconSm: LoadIconW(ptr::null_mut(), IDI_APPLICATION),
			};

			RegisterClassExW(&wc)
		};
		
		if self.class_atom == 0 {
			panic!("Couldn't register class");
		};
	}

	pub fn create_window(&mut self, window_title: &Vec<u16>) {
		let hmenu = unsafe {
			LoadMenuW(self.instance, ID_MENU as WORD as ULONG_PTR as LPWSTR)
		};
		
		if hmenu == ptr::null_mut() {
			let error = unsafe {
				kernel32::GetLastError()
			};
			
			panic!("Couldn't create hmenu: {:?}", error);
		};
		
		unsafe {
			CreateWindowExW(
				0, self.class_atom as LPCWSTR, window_title.as_ptr(),
				WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 480, 320,
				ptr::null_mut(), hmenu, self.instance, self as *mut MainWindow as LPVOID)
		};
		
		if self.hwnd == ptr::null_mut() {
			let error = unsafe {
				kernel32::GetLastError()
			};
			
			panic!("Couldn't create window: {:?}", error);
		};
	}

	fn create_edit(&mut self) {
		let edit_class: Vec<u16> = convert_string("EDIT");
		let blank: Vec<u16> = convert_string("");
		
		assert_ne!(self.hwnd, ptr::null_mut());
		
		self.edit = unsafe {
			CreateWindowExW(WS_EX_CLIENTEDGE, edit_class.as_ptr(), blank.as_ptr(),
				WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL,
				CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, 
				self.hwnd, IDC_EDIT as HMENU, self.instance, ptr::null_mut())
		};
		
		if self.edit == ptr::null_mut() {
			panic!("Couldn't create edit {}", unsafe { kernel32::GetLastError() });
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
				self.hwnd, IDC_TOOLBAR as HMENU, self.instance, ptr::null_mut())
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
			self.status =
				CreateWindowExW(0, statusbar_class.as_ptr(), ptr::null_mut(),
					WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP, 0, 0, 0, 0,
					self.hwnd, IDC_MAIN_STATUS as HMENU, self.instance, ptr::null_mut());
		
			if self.status == ptr::null_mut() {
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
	
	pub fn new() -> MainWindow {
		let class_name: Vec<u16> = convert_string("Rustpad");
		let window_title: Vec<u16> = convert_string("Rustpad");
		
		let mut window = MainWindow {
			hwnd: 0 as HWND,
			edit: 0 as HWND,
			toolbar: 0 as HWND,
			status: 0 as HWND,
			class_atom: 0,
			instance: MainWindow::get_current_instance_handle(),
		};
	
		window.register_window_class(&class_name);
		
		window.create_window(&window_title);
		
		window
	}
}