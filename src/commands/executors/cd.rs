use std::{env, fs};
use std::path::PathBuf;

pub fn execute_cd(args: &[String]) {
    if args.is_empty() {
        return;
    }
    let target = &args[0];
    let target_dir = match resolve_strict_path(target) {
        Some(path) => path,
        None => return,
    };
    if !target_dir.is_dir() {
        eprintln!("cd: {}: Not a directory", target);
        return;
    }
    if let Err(e) = env::set_current_dir(target_dir) {
        eprintln!("cd: {}: {}", target, e);
    }
}

fn resolve_strict_path(path: &str) -> Option<PathBuf> {
    // canonicalize automatically resolves relative to the current working dir,
    // removes "." and "..", and resolves symlinks.
    match fs::canonicalize(path) {
        Ok(path) => Some(path),
        Err(_) => {
            eprintln!("cd: {}: No such file or directory", path);
            None
        },
    }
}