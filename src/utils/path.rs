use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn get_os_paths() -> Option<Vec<PathBuf>> {
    env::var_os("PATH").map(|os_path| env::split_paths(&os_path).collect())
}

pub fn is_file_executable(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    } else {
        false
    }
}