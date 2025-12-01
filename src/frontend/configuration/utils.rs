use crate::frontend::NODE_DATA_DIRS;
use std::io;
use std::path::PathBuf;
use tokio::task;

/// Calculate the total size of known node data directories only
pub(in crate::frontend) async fn calculate_node_data_size(path: PathBuf) -> io::Result<u64> {
    task::spawn_blocking(move || {
        let mut total_size = 0u64;
        for dir_name in NODE_DATA_DIRS {
            let dir_path = path.join(dir_name);
            if dir_path.exists() {
                total_size += fs_extra::dir::get_size(&dir_path)
                    .map_err(|e| io::Error::other(e.to_string()))?;
            }
        }
        Ok(total_size)
    })
    .await
    .map_err(|e| io::Error::other(e.to_string()))?
}

/// Get available space on the volume containing the given path
pub(in crate::frontend) async fn get_available_space(path: PathBuf) -> io::Result<u64> {
    task::spawn_blocking(move || fs4::available_space(&path))
        .await
        .map_err(|e| io::Error::other(e.to_string()))?
}

pub(super) async fn is_directory_writable(path: PathBuf) -> bool {
    if path == PathBuf::new() {
        return false;
    }

    task::spawn_blocking(move || {
        if path.exists() {
            // Try to create a temporary file to check if path is writable
            tempfile::tempfile_in(path).is_ok()
        } else {
            // Try to create a temporary file in parent directory to check if path is writable, and
            // it would be possible to create a parent directory later
            if let Some(parent) = path.parent() {
                tempfile::tempfile_in(parent).is_ok()
            } else {
                false
            }
        }
    })
    .await
    .unwrap_or_default()
}
