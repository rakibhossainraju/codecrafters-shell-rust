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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // `PATH` is process-global, so serialize every test that touches it.
    // (nextest still runs each test in its own process, but keeping this
    // guard makes the module safe to run under a plain `cargo test` too.)
    static PATH_LOCK: Mutex<()> = Mutex::new(());

    fn make_executable(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    fn make_non_executable(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "not executable").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn is_file_executable_detects_the_executable_bit() {
        let dir = TempDir::new().unwrap();
        let exe = make_executable(dir.path(), "runme");
        let not_exe = make_non_executable(dir.path(), "readonly");

        assert!(is_file_executable(&exe));
        assert!(!is_file_executable(&not_exe));
        assert!(!is_file_executable(&dir.path().join("does-not-exist")));
        // A directory is never "executable" in this sense, even with +x on it.
        assert!(!is_file_executable(&dir.path().to_path_buf()));
    }

    #[test]
    fn get_executable_path_finds_first_match_in_path_order() {
        let _guard = PATH_LOCK.lock().unwrap();
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        make_executable(dir_a.path(), "mytool");
        make_executable(dir_b.path(), "mytool");
        make_executable(dir_b.path(), "onlyinb");

        let joined = env::join_paths([dir_a.path(), dir_b.path()]).unwrap();
        let original = env::var_os("PATH");
        unsafe { env::set_var("PATH", &joined) };

        let found = get_executable_path("mytool").expect("should find mytool");
        assert_eq!(found, dir_a.path().join("mytool"));

        let found_b = get_executable_path("onlyinb").expect("should find onlyinb");
        assert_eq!(found_b, dir_b.path().join("onlyinb"));

        assert!(get_executable_path("does-not-exist-anywhere").is_none());

        match original {
            Some(v) => unsafe { env::set_var("PATH", v) },
            None => unsafe { env::remove_var("PATH") },
        }
    }

    #[test]
    fn get_executable_names_deduplicates_and_sorts() {
        let _guard = PATH_LOCK.lock().unwrap();
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        make_executable(dir_a.path(), "zzz");
        make_executable(dir_a.path(), "aaa");
        make_executable(dir_b.path(), "aaa"); // same name, should dedup
        make_non_executable(dir_a.path(), "not_an_exe");

        let joined = env::join_paths([dir_a.path(), dir_b.path()]).unwrap();
        let original = env::var_os("PATH");
        unsafe { env::set_var("PATH", &joined) };

        let names = get_executable_names();

        match original {
            Some(v) => unsafe { env::set_var("PATH", v) },
            None => unsafe { env::remove_var("PATH") },
        }

        assert_eq!(names.iter().filter(|n| n.as_str() == "aaa").count(), 1);
        assert!(names.contains(&"zzz".to_string()));
        assert!(!names.contains(&"not_an_exe".to_string()));
        // sorted
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
