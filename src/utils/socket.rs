use std::io;

/// Prevent long-lived child processes (notably the ADB server on Windows) from
/// keeping a server port open after scrcpy-mask exits.
#[cfg(windows)]
pub fn disable_inheritance<T: std::os::windows::io::AsRawSocket>(socket: &T) -> io::Result<()> {
    windows_handle::set_inheritable(socket.as_raw_socket(), false)
}

#[cfg(not(windows))]
pub fn disable_inheritance<T>(_socket: &T) -> io::Result<()> {
    Ok(())
}

#[cfg(windows)]
mod windows_handle {
    use std::{ffi::c_void, io, os::windows::io::RawSocket};

    const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetHandleInformation(handle: *mut c_void, mask: u32, flags: u32) -> i32;
    }

    pub(super) fn set_inheritable(socket: RawSocket, inheritable: bool) -> io::Result<()> {
        let flags = if inheritable { HANDLE_FLAG_INHERIT } else { 0 };
        let result =
            unsafe { SetHandleInformation(socket as *mut c_void, HANDLE_FLAG_INHERIT, flags) };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use std::{
        ffi::c_void,
        net::{Ipv4Addr, TcpListener},
        os::windows::io::AsRawSocket,
    };

    use super::*;

    const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetHandleInformation(handle: *mut c_void, flags: *mut u32) -> i32;
    }

    #[test]
    fn clears_listener_inherit_flag() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let socket = listener.as_raw_socket();

        windows_handle::set_inheritable(socket, true).unwrap();
        disable_inheritance(&listener).unwrap();

        let mut flags = 0;
        let get_result = unsafe { GetHandleInformation(socket as *mut c_void, &mut flags) };
        assert_ne!(get_result, 0, "failed to read test socket flags");
        assert_eq!(flags & HANDLE_FLAG_INHERIT, 0);
    }
}
