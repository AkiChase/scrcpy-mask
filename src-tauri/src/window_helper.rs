use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

#[derive(serde::Serialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn get_window_list() -> Vec<WindowInfo> {
    let mut windows_info_vec: Vec<WindowInfo> = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_window),
            LPARAM(&mut windows_info_vec as *mut Vec<WindowInfo> as isize),
        )
        .unwrap_or(()); // use unwrap_or to avoid panic
    };
    windows_info_vec
}

extern "system" fn enum_window(window: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let windows_info_vec = &mut *(lparam.0 as *mut Vec<WindowInfo>);
        // &mut *(lparam as *mut Vec<WindowInfo>);

        let mut title: [u16; 512] = [0; 512];
        let len = GetWindowTextW(window, &mut title);
        let title = String::from_utf16_lossy(&title[..len as usize]);

        let mut info = WINDOWINFO {
            cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
            ..Default::default()
        };
        GetWindowInfo(window, &mut info).unwrap();

        let width = info.rcClient.right - info.rcClient.left;
        let height = info.rcClient.bottom - info.rcClient.top;

        if width > 100 && height > 100 && info.dwStyle.contains(WS_VISIBLE) {
            windows_info_vec.push(WindowInfo {
                hwnd: window.0,
                title: if title.is_empty() {
                    "(Untitled)".to_string()
                } else {
                    title
                },
                x: info.rcClient.left,
                y: info.rcClient.top,
                width,
                height,
            });
        }

        true.into()
    }
}

pub fn get_window_control_list(hwnd: isize) -> Vec<WindowInfo> {
    let mut controls_info_vec: Vec<WindowInfo> = Vec::new();

    unsafe {
        EnumChildWindows(
            HWND(hwnd),
            Some(enum_window),
            LPARAM(&mut controls_info_vec as *mut Vec<WindowInfo> as isize),
        ); // use unwrap_or to avoid panic
    }
    controls_info_vec
}
