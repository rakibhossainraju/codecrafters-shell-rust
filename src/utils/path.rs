use std::env;
use std::path::PathBuf;

pub fn get_os_paths() -> Option<Vec<PathBuf>> {
    env::var_os("PATH").map(|os_path| env::split_paths(&os_path).collect())
}
