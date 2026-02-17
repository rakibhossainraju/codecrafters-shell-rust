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

pub fn get_executable_path(cmd_name: &str) -> Option<PathBuf> {
    if let Some(paths) = get_os_paths() {
        for path_str in paths {
            let full_path = path_str.join(cmd_name);
            if is_file_executable(&full_path) {
                return Some(full_path);
            }
        }
    }
    None
}
