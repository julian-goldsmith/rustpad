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

const IDC_EDIT: i32 = 101;
const IDC_TOOLBAR: i32 = 102;
const IDC_MAIN_STATUS: i32 = 104;

const ID_FILE_EXIT: i32 = 9001;
const ID_FILE_NEW: i32 = 9002;
const ID_FILE_OPEN: i32 = 9003;
const ID_FILE_SAVEAS: i32 = 9004;

fn convert_string(string: &str) -> Vec<u16> {
	OsStr::new(string).encode_wide().chain(once(0)).collect()
}

pub struct MainWindow {
	pub hwnd: HWND,
	hinstance: HINSTANCE,
}

static mut MAIN_WINDOW_INSTANCE: Option<MainWindow> = None;

impl MainWindow {
	unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		let mut main_window = if let Some(ref mut main_window) = MAIN_WINDOW_INSTANCE {
			main_window
		} else {
			return 0;
		};
		
		match msg {
			WM_CLOSE => { DestroyWindow(hwnd); },
			WM_DESTROY => { PostQuitMessage(0); },
			WM_CREATE => { main_window.populate_window(); },
			WM_SIZE => { main_window.resize(); },
			WM_COMMAND => {
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
	}

	unsafe fn open_file(&mut self) {
		let mut ofn: OPENFILENAMEW = mem::zeroed();
		let mut filename = [0 as u16; 1024];
		let filter_text = convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
		let default_ext = convert_string("txt");
		
		ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
		ofn.hwndOwner = self.hwnd;
		ofn.lpstrFilter = filter_text.as_ptr();
		ofn.lpstrFile = filename.as_mut_ptr();
		ofn.nMaxFile = filename.len() as u32;
		ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;
		ofn.lpstrDefExt = default_ext.as_ptr();
		
		if GetOpenFileNameW(&mut ofn) != FALSE {
			let edit = GetDlgItem(self.hwnd, IDC_EDIT);
			
			// load edit
			let qqq: Vec<u16> = filename.iter().take_while(|c| **c != 0).map(|c| *c).collect();
			let s = OsString::from_wide(&qqq[0..]).to_string_lossy().into_owned();
			let mut file = File::open(&Path::new(&s)).unwrap();
			
			let mut data = String::new();
			
			file.read_to_string(&mut data).expect("Read file");
			
			let oss = convert_string(&data);
			SendMessageW(edit, WM_SETTEXT, 0, oss.as_ptr() as LPARAM);
		}
	}

	unsafe fn save_file(&mut self) {
		let mut ofn: OPENFILENAMEW = mem::zeroed();
		let mut filename = [0 as u16; 1024];
		let filter_text = convert_string("Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0");
		let default_ext = convert_string("txt");
		
		ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
		ofn.hwndOwner = self.hwnd;
		ofn.lpstrFilter = filter_text.as_ptr();
		ofn.lpstrFile = filename.as_mut_ptr();
		ofn.nMaxFile = filename.len() as u32;
		ofn.Flags = OFN_EXPLORER | OFN_HIDEREADONLY;
		ofn.lpstrDefExt = default_ext.as_ptr();
		
		if GetSaveFileNameW(&mut ofn) != FALSE {
			let edit = GetDlgItem(self.hwnd, IDC_EDIT);
			
			let oss = [0 as u16; 4096];
			SendMessageW(edit, WM_GETTEXT, oss.len() as u64, oss.as_ptr() as LPARAM);
			
			// load edit
			let qqq: Vec<u16> = filename.iter().take_while(|c| **c != 0).map(|c| *c).collect();
			let s = OsString::from_wide(&qqq[0..]).to_string_lossy().into_owned();
			let mut file = File::create(&Path::new(&s)).unwrap();
			
			let qqq2: Vec<u16> = oss.iter().take_while(|c| **c != 0).map(|c| *c).collect();
			let file_text = OsString::from_wide(&qqq2[0..]).to_string_lossy().into_owned();
			
			file.write_all(&file_text.as_bytes()).expect("Write file");
		}
	}

	unsafe fn clear_text(&mut self) {
		let blank: Vec<u16> = convert_string("");
		let edit = GetDlgItem(self.hwnd, IDC_EDIT);
		
		SendMessageW(edit, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
	}

	unsafe fn resize(&mut self) {
		let mut rect: RECT = mem::uninitialized();
		GetClientRect(self.hwnd, &mut rect);
		
		let edit = GetDlgItem(self.hwnd, IDC_EDIT);
		let tool = GetDlgItem(self.hwnd, IDC_TOOLBAR);
		let status = GetDlgItem(self.hwnd, IDC_MAIN_STATUS);
		
		SendMessageW(status, WM_SIZE, 0, 0);
		SendMessageW(tool, TB_AUTOSIZE, 0, 0);
		
		let mut tool_rect: RECT = mem::uninitialized();
		GetWindowRect(tool, &mut tool_rect);
		
		let mut status_rect: RECT = mem::uninitialized();
		GetWindowRect(status, &mut status_rect);
		
		let tool_height = tool_rect.bottom - tool_rect.top;
		let status_height = status_rect.bottom - status_rect.top;
		let edit_height = rect.bottom - tool_height - status_height;
		
		SetWindowPos(edit, ptr::null_mut(), 0, tool_height, rect.right, edit_height, SWP_NOZORDER);
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

	pub fn create_window(instance: HINSTANCE, class_name: &Vec<u16>, window_title: &Vec<u16>) -> HWND {
		let hwnd = unsafe {
			CreateWindowExW(
				0, class_name.as_ptr(), window_title.as_ptr(),
				WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 480, 320,
				ptr::null_mut(), ptr::null_mut(), instance, ptr::null_mut())
		};
			
		if hwnd == ptr::null_mut() {
			panic!("Couldn't create window");
		};
		
		hwnd
	}

	fn create_edit(&mut self) -> HWND {
		let edit_class: Vec<u16> = convert_string("EDIT");
		let blank: Vec<u16> = convert_string("");
		
		let edit = unsafe {
			CreateWindowExW(WS_EX_CLIENTEDGE, edit_class.as_ptr(), blank.as_ptr(),
				WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL,
				CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, 
				self.hwnd, IDC_EDIT as HMENU, self.hinstance, ptr::null_mut())
		};
		
		if edit == ptr::null_mut() {
			panic!("Couldn't create edit");
		};
		
		unsafe {
			let font = GetStockObject(DEFAULT_GUI_FONT) as HFONT;
			SendMessageW(edit, WM_SETFONT, font as WPARAM, 0);
		
			SendMessageW(edit, WM_SETTEXT, 0, blank.as_ptr() as LPARAM);
		};
		
		edit
	}

	fn create_menu(&mut self) {
		let exit_string = convert_string("E&xit");
		let file_string = convert_string("&File");
		let new_string = convert_string("&New");
		let open_string = convert_string("&Open");
		let save_as_string = convert_string("Save &As");
		
		let menu = unsafe {
			let menu = CreateMenu();
			
			let sub_menu = CreatePopupMenu();
			
			AppendMenuW(sub_menu, MF_STRING, ID_FILE_NEW as u64, new_string.as_ptr());
			AppendMenuW(sub_menu, MF_SEPARATOR, 0, ptr::null());
			
			AppendMenuW(sub_menu, MF_STRING, ID_FILE_OPEN as u64, open_string.as_ptr());
			AppendMenuW(sub_menu, MF_STRING, ID_FILE_SAVEAS as u64, save_as_string.as_ptr());
			AppendMenuW(sub_menu, MF_SEPARATOR, 0, ptr::null());

			AppendMenuW(sub_menu, MF_STRING, ID_FILE_EXIT as u64, exit_string.as_ptr());
			
			AppendMenuW(menu, MF_STRING | MF_POPUP, sub_menu as u64, file_string.as_ptr());
			
			SetMenu(self.hwnd, menu);
			
			menu
		};
		
		if menu == ptr::null_mut() {
			panic!("Couldn't create menu");
		};
	}

	fn create_toolbar(&mut self) -> HWND {
		let toolbar_name = convert_string("ToolbarWindow32");
		
		let toolbar = unsafe {
			CreateWindowExW(0, toolbar_name.as_ptr(), ptr::null_mut(), WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, 
				self.hwnd, IDC_TOOLBAR as HMENU, self.hinstance, ptr::null_mut())
		};
		
		if toolbar == ptr::null_mut() {
			panic!("Couldn't create toolbar");
		};
		
		unsafe {
			SendMessageW(toolbar, TB_BUTTONSTRUCTSIZE, mem::size_of::<commctrl::TBBUTTON>() as WPARAM, 0);
		};
		
		let tbab = TBADDBITMAP {
			hInst: -1 as i64 as HINSTANCE,//HINST_COMMCTRL,
			nID: IDB_STD_SMALL_COLOR,
		};
		
		unsafe {
			SendMessageW(toolbar, TB_ADDBITMAP, 0, &tbab as *const TBADDBITMAP as LPARAM)
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
			SendMessageW(toolbar, TB_ADDBUTTONSW, tbb.len() as u64, tbb.as_ptr() as LPARAM);
		};
		
		toolbar
	}

	fn create_status(&mut self) -> HWND {
		let statusbar_class = convert_string("msctls_statusbar32");
		
		let status = unsafe {
			CreateWindowExW(0, statusbar_class.as_ptr(), ptr::null_mut(),
				WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP, 0, 0, 0, 0,
				self.hwnd, IDC_MAIN_STATUS as HMENU, self.hinstance, ptr::null_mut())
		};
		
		if status == ptr::null_mut() {
			panic!("Couldn't create status");
		};
		
		status
	}

	fn populate_window(&mut self) {		
		self.create_menu();
		self.create_edit();
		self.create_toolbar();
		self.create_status();
	}

	fn get_current_instance_handle() -> HINSTANCE {
		unsafe {
			kernel32::GetModuleHandleW(ptr::null_mut())
		}
	}
	
	fn new() -> MainWindow {
		let class_name: Vec<u16> = OsStr::new("myWindowClass").encode_wide().chain(once(0)).collect();
		let window_title: Vec<u16> = OsStr::new("Test window").encode_wide().chain(once(0)).collect();
	
		let instance = MainWindow::get_current_instance_handle();
	
		MainWindow::register_window_class(instance, &class_name);
		
		let hwnd = MainWindow::create_window(instance, &class_name, &window_title);
		
		MainWindow {
			hwnd: hwnd,
			hinstance: instance,
		}
	}
	
	pub fn get_instance() -> &'static MainWindow {
		unsafe {
			if let Some(ref mut main_window) = MAIN_WINDOW_INSTANCE {
				main_window
			} else {
				MAIN_WINDOW_INSTANCE = Some(MainWindow::new());
				MainWindow::get_instance()
			}
		}
	}
}