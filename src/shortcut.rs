#[allow(unused_imports)]
use std::ffi::CString;
use std::path::PathBuf;
#[allow(unused_imports)]
use std::process::Command;

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteA(
        hwnd: isize,
        lpOperation: *const i8,
        lpFile: *const i8,
        lpParameters: *const i8,
        lpDirectory: *const i8,
        nShowCmd: i32,
    ) -> isize;
}

pub fn open_folder(path: PathBuf) {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path.to_str().unwrap())
            .spawn()
            .expect("Failed to Open Folder");
    }
    #[cfg(target_os = "windows")]
    {
        let operation = "open\0";
        let os_str = path.as_os_str();
        let c_str =
            CString::new(os_str.to_string_lossy().into_owned()).expect("CString::new failed");
        unsafe {
            ShellExecuteA(
                0,
                operation.as_ptr() as *const i8,
                c_str.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path.to_str().unwrap())
            .spawn()
            .expect("Failed to Open Folder");
    }
}
