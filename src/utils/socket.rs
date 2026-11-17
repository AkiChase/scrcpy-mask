use std::{io, net::SocketAddrV4};

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

/// Bind a TCP listener that stays private to this process on Windows.
///
/// `tokio::net::TcpListener::bind` creates its socket through mio's raw
/// `socket()` call, which is inheritable by default on Windows. Clearing
/// `HANDLE_FLAG_INHERIT` afterwards (`disable_inheritance`) is not enough for
/// accepted connections: whether `accept()` returns an inheritable socket
/// depends on the listener's *creation-time* inheritability, which cannot be
/// changed later.
///
/// Binding through `std::net` creates the socket with
/// `WSA_FLAG_NO_HANDLE_INHERIT`, so both the listener and every socket it
/// accepts are non-inheritable, and child processes (notably the long-lived
/// ADB server) can no longer keep the port open after scrcpy-mask exits.
pub fn bind_listener(addr: SocketAddrV4) -> io::Result<tokio::net::TcpListener> {
    // Binding a numeric address is a local syscall, so it is fine to do it
    // synchronously instead of going through `spawn_blocking`.
    let std_listener = std::net::TcpListener::bind(addr)?;
    std_listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(std_listener)?;
    // Belt and braces: `std::net` already creates the socket non-inheritable,
    // but make sure the flag is cleared no matter what.
    disable_inheritance(&listener)
        .map_err(|e| io::Error::other(format!("failed to clear HANDLE_FLAG_INHERIT: {e}")))?;
    Ok(listener)
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
        net::{Ipv4Addr, SocketAddrV4, TcpListener},
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

    /// `accept()` returns an inheritable socket when the listener was created
    /// inheritable, so binding through `std::net` (which creates the socket
    /// non-inheritable) is what keeps accepted connections private too.
    #[test]
    fn accepted_sockets_are_not_inheritable() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let listener = bind_listener(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
                let addr = listener.local_addr().unwrap();
                let client = std::net::TcpStream::connect(addr).unwrap();

                let (socket, _) = listener.accept().await.unwrap();

                let mut flags = 0;
                let get_result = unsafe {
                    GetHandleInformation(socket.as_raw_socket() as *mut c_void, &mut flags)
                };
                assert_ne!(get_result, 0, "failed to read accepted socket flags");
                assert_eq!(flags & HANDLE_FLAG_INHERIT, 0);

                drop(client);
            });
    }
}
