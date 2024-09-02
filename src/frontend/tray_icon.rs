// TODO: `tray-icon` crate on Linux pulls GTK3 legacy dependency and is undesirable because of it:
//  https://github.com/tauri-apps/tray-icon/issues/107
//  If `ksni` works well, we should upstream it into `tray-icon` itself, then we can use a single
//  crate again.
#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod windows_macos;

use crate::frontend::ICON;
use image::{ImageBuffer, Rgba};
#[cfg(all(unix, not(target_os = "macos")))]
pub(super) use unix::spawn;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) use windows_macos::spawn;

fn load_icon() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
        .expect("Statically correct image; qed")
        .to_rgba8()
}
