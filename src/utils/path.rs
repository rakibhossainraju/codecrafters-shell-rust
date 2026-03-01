use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn get_os_paths() -> Option<Vec<PathBuf>> {
    env::var_os("PATH").map(|os_path| env::split_paths(&os_path).collect())
}

// pub fn get_relative_path(path: &str) -> Option<String> {
//     env::current_dir().ok()?.join(path).to_str().map(String::from)
// }

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

pub fn get_executables_paths() -> Vec<PathBuf> {
    get_os_paths()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|e| e.path())
        .filter(is_file_executable)
        .collect()
}

pub fn get_executable_names() -> Vec<String> {
    let mut names: Vec<String> = get_executables_paths()
        .into_iter()
        .filter_map(|path| {
            path.file_name()?
                .to_str()
                .map(String::from)
        })
        .collect();
    
    names.sort();
    names.dedup();
    names
}
