extern crate comctl32;
extern crate comrak;
extern crate gdi32;
extern crate kernel32;
extern crate typed_arena;
extern crate user32;
extern crate winapi;

mod markdowneditcontrol;

use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use user32::*;
use winapi::*;
use kernel32::LoadLibraryW;

const IDC_EDIT: i32 = 101;
const IDC_TOOLBAR: i32 = 102;
const IDC_MAIN_STATUS: i32 = 104;

const ID_FILE_EXIT: i32 = 9001;

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	match msg {
		WM_CLOSE => { DestroyWindow(hwnd); },
		WM_DESTROY => { PostQuitMessage(0); },
		WM_CREATE => { populate_window(hwnd); },
		WM_SIZE => { resize(hwnd); },
		WM_COMMAND => {
			match LOWORD(w_param as u32) as i32 {
				ID_FILE_EXIT=> PostMessageW(hwnd, WM_CLOSE, 0, 0),
				_ => 0,
			};
		},
		_ => { return DefWindowProcW(hwnd, msg, w_param, l_param); },
	};
	0
}

unsafe fn resize(hwnd: HWND) {
	let mut rect: RECT = mem::uninitialized();
	GetClientRect(hwnd, &mut rect);
	
	let edit = GetDlgItem(hwnd, IDC_EDIT);
	let tool = GetDlgItem(hwnd, IDC_TOOLBAR);
	let status = GetDlgItem(hwnd, IDC_MAIN_STATUS);
	
	SendMessageW(tool, TB_AUTOSIZE, 0, 0);
	SendMessageW(status, WM_SIZE, 0, 0);
	
	let mut tool_rect: RECT = mem::uninitialized();
	GetWindowRect(tool, &mut tool_rect);
	
	let mut status_rect: RECT = mem::uninitialized();
	GetWindowRect(status, &mut status_rect);
	
	let tool_height = tool_rect.bottom - tool_rect.top;
	let status_height = status_rect.bottom - status_rect.top;
	let edit_height = rect.bottom - tool_height - status_height;
	
	SetWindowPos(edit, ptr::null_mut(), 0, tool_height, rect.right, edit_height, SWP_NOZORDER);
}

fn register_window_class(h_instance: HINSTANCE, class_name: &Vec<u16>) -> ATOM {
	let atom = unsafe {
		let wc = WNDCLASSEXW {
			cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
			style: 0,
			lpfnWndProc: Some(wndproc),
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

fn create_window(h_instance: HINSTANCE, class_name: &Vec<u16>, window_title: &Vec<u16>) -> HWND {
	let hwnd = unsafe {
		CreateWindowExW(
			0, class_name.as_ptr(), window_title.as_ptr(),
			WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 480, 320,
			ptr::null_mut(), ptr::null_mut(), h_instance, ptr::null_mut())
	};
		
	if hwnd == ptr::null_mut() {
		panic!("Couldn't create window");
	};
	hwnd
}

fn create_edit(parent_hwnd: HWND) -> HWND {
	let edit: Vec<u16> = OsStr::new("EDIT").encode_wide().chain(once(0)).collect();
	let blank: Vec<u16> = OsStr::new("").encode_wide().chain(once(0)).collect();
	let instance = get_current_instance_handle();
	
	let edit = unsafe {
		CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), blank.as_ptr(),
			WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_HSCROLL | ES_MULTILINE | ES_AUTOHSCROLL | ES_AUTOVSCROLL | WS_BORDER | WS_TABSTOP,
			0, 0, 100, 100, parent_hwnd, IDC_EDIT as HMENU, instance, ptr::null_mut())
	};
	
	if edit == ptr::null_mut() {
		panic!("Couldn't create edit");
	};
	
	edit
}

fn create_toolbar(parent_hwnd: HWND) -> HWND {
	let instance = get_current_instance_handle();
	let toolbar_name: Vec<u16> = OsStr::new("ToolbarWindow32").encode_wide().chain(once(0)).collect();
	
	let toolbar = unsafe {
		CreateWindowExW(0, toolbar_name.as_ptr(), ptr::null_mut(), WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, 
			parent_hwnd, IDC_TOOLBAR as HMENU, instance, ptr::null_mut())
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
	
	let tbb: [TBBUTTON; 1] = [
		TBBUTTON {
			iBitmap: STD_FILENEW,
			idCommand: ID_FILE_EXIT,
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

fn create_menu(parent_hwnd: HWND) {
	let exit_string: Vec<u16> = OsStr::new("E&xit").encode_wide().chain(once(0)).collect();
	let file_string: Vec<u16> = OsStr::new("&File").encode_wide().chain(once(0)).collect();
	
	let menu = unsafe {
		let menu = CreateMenu();
		
		let sub_menu = CreatePopupMenu();
		AppendMenuW(sub_menu, MF_STRING, ID_FILE_EXIT as u64, exit_string.as_ptr());
		AppendMenuW(menu, MF_STRING | MF_POPUP, sub_menu as u64, file_string.as_ptr());
		
		SetMenu(parent_hwnd, menu);
		
		menu
	};
	
	if menu == ptr::null_mut() {
		panic!("Couldn't create menu");
	};
}

fn create_status(parent_hwnd: HWND) -> HWND {
	let statusbar_class: Vec<u16> = OsStr::new("msctls_statusbar32").encode_wide().chain(once(0)).collect();
	let instance = get_current_instance_handle();
		
	let status = unsafe {
		CreateWindowExW(0, statusbar_class.as_ptr(), ptr::null_mut(),
			WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP, 0, 0, 0, 0,
			parent_hwnd, IDC_MAIN_STATUS as HMENU, instance, ptr::null_mut())
	};
	
	if status == ptr::null_mut() {
		panic!("Couldn't create status");
	};
	
	status
}

fn populate_window(hwnd: HWND) {
	create_menu(hwnd);
	create_edit(hwnd);
	create_toolbar(hwnd);
	create_status(hwnd);
}

fn get_current_instance_handle() -> HINSTANCE {
	unsafe {
		kernel32::GetModuleHandleW(ptr::null_mut())
	}
}

fn main() {
/*
	let arena = Arena::new();

	let root = parse_document(
		&arena,
		"This is my input.\n\n1. Also my input.\n2. Certainly my input.\n",
		&ComrakOptions::default());

	fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F, depth: usize) //'
		where F : Fn(&'a AstNode<'a>) {
		f(node);
		let spaces: String = repeat(' ').take(depth * 2).collect();
		println!("{}Node {:?}", &spaces, &node.data.borrow().value);
		for c in node.children() {
			iter_nodes(c, f, depth + 1);
		}
	}

	iter_nodes(root, &|node| {
		
	}, 0);

	return;
*/

	let richedit_dll: Vec<u16> = OsStr::new("Riched20.dll").encode_wide().chain(once(0)).collect();
	
	unsafe {
		LoadLibraryW(richedit_dll.as_ptr());
	};

	let class_name: Vec<u16> = OsStr::new("myWindowClass").encode_wide().chain(once(0)).collect();
	let window_title: Vec<u16> = OsStr::new("Test window").encode_wide().chain(once(0)).collect();
	
	unsafe {
		comctl32::InitCommonControls();
	};
	
	markdowneditcontrol::register();
	
	let h_instance = get_current_instance_handle();
	
	register_window_class(h_instance, &class_name);
	
	let hwnd = create_window(h_instance, &class_name, &window_title);
	
	unsafe {	
		ShowWindow(hwnd, SW_SHOW);
		UpdateWindow(hwnd);
	};
	
	unsafe {
		let mut msg: MSG = mem::uninitialized();
		
		while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
			DispatchMessageW(&msg);
		};
	};
	
	
	markdowneditcontrol::unregister();
}