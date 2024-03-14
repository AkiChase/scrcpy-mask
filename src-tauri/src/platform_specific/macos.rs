#[test]
fn test() {
    window_helper::get_window_list();
}

pub mod window_helper {
    use std::ffi::{c_int, c_void, CString};

    use core_foundation::{
        base::TCFType,
        dictionary::{CFDictionaryGetValue, __CFDictionary},
        number::{kCFNumberIntType, CFNumberGetValue, CFNumberRef},
        string::{kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringGetCString},
    };
    use core_graphics::display::*;

    #[derive(serde::Serialize)]
    pub struct WindowInfo {
        pub hwnd: i32,
        pub title: String,
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    // I have almost no prior knowledge of the macOS API, and there are very few Rust examples about macOS.
    // Writing this code is like playing the Shawshank Redemption.
    // Thanks for the company of ChatGPT.
    pub fn get_window_list() -> Vec<WindowInfo> {
        let mut window_info_vec: Vec<WindowInfo> = Vec::new();

        // CGWindowListOption is a bitmask, combine the flags with bitwise OR
        const OPTIONS: CGWindowListOption = kCGWindowListOptionOnScreenOnly;
        // No need to specify the type or use 'as'; CFArrayRef is the return type from CGWindowListCopyWindowInfo
        let window_list_info = unsafe { CGWindowListCopyWindowInfo(OPTIONS, kCGNullWindowID) };
        // Don't use const here, CFArrayGetCount returns CFIndex (long)
        let count = unsafe { CFArrayGetCount(window_list_info) };
        println!("{}", count);

        for i in 0..count {
            let dic_ref = unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };

            let mut window_id: c_int = 0;
            let num_ref = get_cfdict_val("kCGWindowNumber", dic_ref);
            unsafe {
                CFNumberGetValue(
                    num_ref as CFNumberRef,
                    kCFNumberIntType,
                    &mut window_id as *mut c_int as *mut c_void,
                )
            };

            let name_ref = get_cfdict_val("kCGWindowOwnerName", dic_ref);
            let owner_name = cf_ref_to_string(name_ref);

            let val = get_cfdict_val("kCGWindowBounds", dic_ref);
            let rect_cf =
                unsafe { CFDictionary::wrap_under_get_rule(val as *const __CFDictionary) };
            let rect = CGRect::from_dict_representation(&rect_cf).unwrap();

            
            window_info_vec.push(WindowInfo{
                hwnd: window_id,
                title: owner_name,
                x: rect.origin.x as i32,
                y: rect.origin.y as i32,
                width: rect.size.width as i32,
                height: rect.size.height as i32,
            })
        }

        window_info_vec
    }

    pub fn get_window_control_list(hwnd: isize) -> Vec<WindowInfo> {
        println!("{hwnd}");
        Vec::new()
    }

    fn get_cfdict_val(key: &str, dic_ref: *const __CFDictionary) -> *const c_void {
        let key_ptr = str_to_cf_key_ptr(key);
        unsafe { CFDictionaryGetValue(dic_ref, key_ptr) }
    }

    fn str_to_cf_key_ptr(key: &str) -> *const c_void {
        let c_key = CString::new(key).unwrap();
        let cf_key = unsafe {
            CFStringCreateWithCString(std::ptr::null(), c_key.as_ptr(), kCFStringEncodingUTF8)
        };

        // cf_key is a CFStringRef, which is a type alias to *const __CFString
        // We transmute it into *const c_void since that is what CFDictionaryGetValueIfPresent wants
        unsafe { std::mem::transmute(cf_key) }
    }

    fn cf_ref_to_string(cf_ref: *const c_void) -> String {
        let cf_ref = cf_ref as core_foundation::string::CFStringRef;
        // let c_ptr = unsafe { CFStringGetCStringPtr(cf_ref, kCFStringEncodingUTF8) };
        let mut buffer: [i8; 1024] = [0; 1024];
        if unsafe {
            CFStringGetCString(cf_ref, buffer.as_mut_ptr(), 1024, kCFStringEncodingUTF8) != 0
        } {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
            let rust_str = c_str.to_str().unwrap();
            rust_str.into()
        } else {
            String::from("Nameless")
        }
    }
}
