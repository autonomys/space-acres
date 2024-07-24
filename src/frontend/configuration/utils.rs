use std::path::PathBuf;
use tokio::task;

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
