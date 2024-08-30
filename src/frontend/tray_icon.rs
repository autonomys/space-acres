#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod windows_macos;

#[cfg(all(unix, not(target_os = "macos")))]
pub(super) use unix::TrayIcon;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) use windows_macos::TrayIcon;
