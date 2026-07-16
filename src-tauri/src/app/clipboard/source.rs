use std::path::Path;

pub fn current_application() -> Option<String> {
    platform::current_application()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

pub fn comparable_application_name(value: &str) -> String {
    let filename = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .trim()
        .to_lowercase();
    filename
        .strip_suffix(".exe")
        .or_else(|| filename.strip_suffix(".app"))
        .unwrap_or(&filename)
        .to_string()
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;

    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    #[link(name = "user32")]
    extern "system" {
        fn GetClipboardOwner() -> *mut c_void;
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(window: *mut c_void, process_id: *mut u32) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit_handle: i32, process_id: u32) -> *mut c_void;
        fn QueryFullProcessImageNameW(
            process: *mut c_void,
            flags: u32,
            name: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    pub fn current_application() -> Option<String> {
        // The clipboard owner is the most accurate source after a copy. Some
        // applications release ownership immediately, so fall back to the
        // foreground window in that case.
        let window = unsafe {
            let owner = GetClipboardOwner();
            if owner.is_null() {
                GetForegroundWindow()
            } else {
                owner
            }
        };
        if window.is_null() {
            return None;
        }

        let mut process_id = 0;
        unsafe { GetWindowThreadProcessId(window, &mut process_id) };
        if process_id == 0 {
            return None;
        }

        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
        if process.is_null() {
            return None;
        }

        let mut buffer = vec![0u16; 32_768];
        let mut size = buffer.len() as u32;
        let result = unsafe {
            let result = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(process);
            result
        };
        if result == 0 || size == 0 {
            return None;
        }

        let path = OsString::from_wide(&buffer[..size as usize]);
        Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use objc2_app_kit::NSWorkspace;

    pub fn current_application() -> Option<String> {
        let application = NSWorkspace::sharedWorkspace().frontmostApplication()?;
        application.localizedName().map(|name| name.to_string())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::ffi::{c_char, c_int, c_long, c_uchar, c_ulong, c_void, CString};
    use std::fs;
    use std::ptr;

    type Display = c_void;
    type Window = c_ulong;
    type Atom = c_ulong;

    #[link(name = "X11")]
    extern "C" {
        fn XOpenDisplay(name: *const c_char) -> *mut Display;
        fn XDefaultRootWindow(display: *mut Display) -> Window;
        fn XInternAtom(display: *mut Display, name: *const c_char, only_if_exists: c_int) -> Atom;
        fn XGetWindowProperty(
            display: *mut Display,
            window: Window,
            property: Atom,
            offset: c_long,
            length: c_long,
            delete: c_int,
            requested_type: Atom,
            actual_type: *mut Atom,
            actual_format: *mut c_int,
            item_count: *mut c_ulong,
            bytes_after: *mut c_ulong,
            value: *mut *mut c_uchar,
        ) -> c_int;
        fn XFree(data: *mut c_void) -> c_int;
        fn XCloseDisplay(display: *mut Display) -> c_int;
    }

    const SUCCESS: c_int = 0;

    pub fn current_application() -> Option<String> {
        // Pure Wayland intentionally returns None: the protocol does not expose
        // the application that owns another client's clipboard selection.
        if std::env::var("DISPLAY").ok()?.trim().is_empty() {
            return None;
        }

        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return None;
            }
            let result = active_process_id(display)
                .and_then(|pid| fs::read_to_string(format!("/proc/{pid}/comm")).ok())
                .map(|name| name.trim().to_string());
            XCloseDisplay(display);
            result
        }
    }

    unsafe fn active_process_id(display: *mut Display) -> Option<u32> {
        let root = XDefaultRootWindow(display);
        let active_atom = intern_atom(display, "_NET_ACTIVE_WINDOW")?;
        let pid_atom = intern_atom(display, "_NET_WM_PID")?;
        let active_window = read_ulong_property(display, root, active_atom)? as Window;
        read_ulong_property(display, active_window, pid_atom).map(|pid| pid as u32)
    }

    unsafe fn intern_atom(display: *mut Display, name: &str) -> Option<Atom> {
        let name = CString::new(name).ok()?;
        let atom = XInternAtom(display, name.as_ptr(), 1);
        (atom != 0).then_some(atom)
    }

    unsafe fn read_ulong_property(
        display: *mut Display,
        window: Window,
        property: Atom,
    ) -> Option<c_ulong> {
        let mut actual_type = 0;
        let mut actual_format = 0;
        let mut item_count = 0;
        let mut bytes_after = 0;
        let mut value = ptr::null_mut();
        let status = XGetWindowProperty(
            display,
            window,
            property,
            0,
            1,
            0,
            0,
            &mut actual_type,
            &mut actual_format,
            &mut item_count,
            &mut bytes_after,
            &mut value,
        );
        if status != SUCCESS || value.is_null() || item_count == 0 {
            if !value.is_null() {
                XFree(value.cast());
            }
            return None;
        }
        let result = if actual_format == 32 {
            Some(*(value as *const c_ulong))
        } else {
            None
        };
        XFree(value.cast());
        result
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {
    pub fn current_application() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_executable_and_bundle_names() {
        assert_eq!(
            comparable_application_name("C:/Windows/notepad.exe"),
            "notepad"
        );
        assert_eq!(comparable_application_name("Notes.app"), "notes");
        assert_eq!(
            comparable_application_name("Visual Studio Code"),
            "visual studio code"
        );
    }
}
