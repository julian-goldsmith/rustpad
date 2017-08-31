use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::mem;
use std::ptr;
use std::io::Write;
use std::panic;
use user32::*;
use winapi::*;
use comdlg32::*;
use kernel32;
use std::fs::File;
use std::path::Path;
use std::slice;
use std::str;
use std::process;
use control::{Control,Edit,Statusbar,Toolbar};
use dialogs;
use util;

const IDC_EDIT: i32 = 101;
const IDC_TOOLBAR: i32 = 102;
const IDC_STATUS: i32 = 104;

const ID_MENU: HMENU = 9000 as HMENU;
const ID_FILE_EXIT: i32 = 9001;
const ID_FILE_NEW: i32 = 9002;
const ID_FILE_OPEN: i32 = 9003;
const ID_FILE_SAVEAS: i32 = 9004;

const TBAB: TBADDBITMAP = TBADDBITMAP {
	hInst: -1 as i64 as HINSTANCE,//HINST_COMMCTRL,
	nID: IDB_STD_SMALL_COLOR,
};

const TBB: [TBBUTTON; 3] = [
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

#[derive(Debug)]
pub struct MainWindow {
	pub hwnd: HWND,
	pub edit: Option<Edit>,
	pub toolbar: Option<Toolbar>,
	pub status: Option<Statusbar>,
	pub instance: HINSTANCE,
	pub class_atom: ATOM,
}

impl Drop for MainWindow {
	fn drop(&mut self) {
	}
}

impl MainWindow {
	unsafe fn get_main_window(hwnd: HWND) -> &'static mut MainWindow {
		(GetWindowLongPtrW(hwnd, 0) as *mut MainWindow).as_mut().unwrap()
	}

	unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		let result = panic::catch_unwind(|| {
			match msg { 
				WM_CLOSE => { DestroyWindow(hwnd); },
				WM_DESTROY => { PostQuitMessage(0); },
				WM_NCCREATE => {
					let cs = l_param as *mut CREATESTRUCTW;
					let mut main_window = ((*cs).lpCreateParams as *mut MainWindow).as_mut().unwrap();
					
					main_window.hwnd = hwnd;
					
					SetWindowLongPtrW(hwnd, 0, main_window as *mut MainWindow as LONG_PTR);
					
					return TRUE as LRESULT; 
				},
				WM_CREATE => {
					let mut main_window = MainWindow::get_main_window(hwnd);
					
					main_window.populate_window();
				},
				WM_SIZE => {
					let mut main_window = MainWindow::get_main_window(hwnd);
					
					main_window.resize();
				},
				WM_COMMAND => {
					let mut main_window = MainWindow::get_main_window(hwnd);
					
					assert_eq!(hwnd, main_window.hwnd);
					
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
		let filename = match dialogs::open_file(self.hwnd) {
			None => return,
			Some(filename) => filename,
		};
		
		let file = kernel32::CreateFileW(
			filename.as_ptr(), GENERIC_READ, FILE_SHARE_READ, ptr::null_mut(),
			OPEN_EXISTING, 0, ptr::null_mut());
			
		if file == INVALID_HANDLE_VALUE {
			let filename_converted = OsString::from_wide(&filename).to_string_lossy().into_owned();
			panic!("Open file \"{}\" failed: {}", filename_converted, kernel32::GetLastError());
		}
		
		let file_size = kernel32::GetFileSize(file, ptr::null_mut());			// right now we don't support files > 4GB
		
		let mut data = kernel32::LocalAlloc(0, file_size as SIZE_T) as LPSTR;
		let mut bytes_read = 0 as DWORD;
		
		// FIXME: we need to load files longer than the buffer
		if kernel32::ReadFile(file, data as LPVOID, file_size as DWORD,
				&mut bytes_read as LPDWORD, ptr::null_mut()) == FALSE {
			panic!("Read failed: {}", kernel32::GetLastError());
		};
		
		kernel32::CloseHandle(file);
		
		let chars = kernel32::LocalAlloc(0, bytes_read as SIZE_T * 2) as LPWSTR;
		if kernel32::MultiByteToWideChar(CP_UTF8, 0, data as LPCSTR, 
				bytes_read as c_int, chars, bytes_read as c_int) == 0 {
			panic!("Character convert failed: {}", kernel32::GetLastError());
		};
		
		kernel32::LocalFree(data as HLOCAL);
		
		let mut edit = self.edit.as_mut().unwrap();
		edit.set_text(slice::from_raw_parts(chars, bytes_read as usize));
		kernel32::LocalFree(chars as HLOCAL);
	}

	unsafe fn save_file(&mut self) {
		let mut ofn: OPENFILENAMEW = mem::zeroed();
		let mut filename_buf = [0 as u16; 1024];
		let filter_text = util::convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
		let default_ext = util::convert_string("txt");
		
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
			
			let edit = self.edit.as_ref().unwrap();
			let text = edit.get_text();
			let file_text = OsString::from_wide(&text).to_string_lossy().into_owned();
			
			file.write_all(&file_text.as_bytes()).expect("Write file");
		}
	}

	unsafe fn clear_text(&mut self) {
		let blank: Vec<u16> = util::convert_string("");
		
		let mut edit = self.edit.as_mut().unwrap();
		edit.set_text(&blank);
	}
	
	unsafe fn get_size(&self) -> RECT{
		let mut rect: RECT = mem::uninitialized();
		GetClientRect(self.hwnd, &mut rect);
		rect
	}

	unsafe fn resize(&mut self) {
		let rect = self.get_size();
		
		let mut toolbar = self.toolbar.as_mut().unwrap();
		let mut status = self.status.as_mut().unwrap();
		toolbar.resize();
		status.resize();
		
		let status_rect = status.get_size();
		let tool_rect: RECT = toolbar.get_size();
		
		let tool_height = tool_rect.bottom - tool_rect.top;
		let status_height = status_rect.bottom - status_rect.top;
		let edit_height = rect.bottom - tool_height - status_height;
		
		let edit = self.edit.as_ref().unwrap();
		SetWindowPos(edit.hwnd, ptr::null_mut(), 0, tool_height, rect.right, edit_height, SWP_NOZORDER);
	}
	
	pub fn show(&self) {
		unsafe {
			ShowWindow(self.hwnd, SW_SHOW);
			UpdateWindow(self.hwnd);
		};
	}

	pub unsafe fn register_window_class(&mut self, class_name: &Vec<u16>) {
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

		self.class_atom = RegisterClassExW(&wc);
		
		if self.class_atom == 0 {
			panic!("Couldn't register class: {:?}", kernel32::GetLastError());
		};
	}

	pub unsafe fn create_window(&mut self, window_title: &Vec<u16>) {
		let hmenu = LoadMenuW(self.instance, ID_MENU as WORD as ULONG_PTR as LPWSTR);
		
		if hmenu == ptr::null_mut() {
			panic!("Couldn't create hmenu: {:?}", kernel32::GetLastError());
		};
		
		CreateWindowExW(
			0, self.class_atom as LPCWSTR, window_title.as_ptr(),
			WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 480, 320,
			ptr::null_mut(), hmenu, self.instance, self as *mut MainWindow as LPVOID);
		
		if self.hwnd == ptr::null_mut() {
			panic!("Couldn't create window: {:?}", kernel32::GetLastError());
		};
	}

	fn populate_window(&mut self) {
		self.edit = Some(Edit::new(self.instance, self.hwnd, IDC_EDIT as HMENU));
		self.toolbar = Some(Toolbar::new(self.instance, self.hwnd, IDC_TOOLBAR as HMENU));
		self.status = Some(Statusbar::new(self.instance, self.hwnd, IDC_STATUS as HMENU));
		
		let toolbar = self.toolbar.as_mut().unwrap();
		toolbar.add_bitmap(&TBAB);
		toolbar.add_buttons(&TBB);
		
		let status = self.status.as_mut().unwrap();
		status.set_text(&util::convert_string("Status bar"));
	}

	fn get_current_instance_handle() -> HINSTANCE {
		unsafe {
			kernel32::GetModuleHandleW(ptr::null_mut())
		}
	}
	
	pub fn initialize(&mut self) {
		self.hwnd = 0 as HWND;
		self.edit = None;
		self.toolbar = None;
		self.status = None;
		self.class_atom = 0;
		self.instance = MainWindow::get_current_instance_handle();
		
		let class_name: Vec<u16> = util::convert_string("Rustpad");
		let window_title: Vec<u16> = util::convert_string("Rustpad");
		
		unsafe {
			self.register_window_class(&class_name);
			
			self.create_window(&window_title);
		};
	}
}