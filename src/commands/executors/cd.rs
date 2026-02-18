use std::{env, fs};

pub fn execute_cd(args: &[String]) {
    // 1. Get the raw path string, defaulting to "~" if empty
    let raw_path = args.first().map(|s| s.as_str()).unwrap_or("~");

    // 2. Handle Home Directory Expansion.
    // We must handle specific cases: "~" alone, or paths starting with "~/"
    let target_str = if raw_path == "~"{
        env::var("HOME").unwrap_or("/".into())
    } else if raw_path.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            // Create a new string: $HOME + /Documents
            // We skip the first character (the ~) from raw_path
            format!("{}{}", home, &raw_path[1..])
        } else {
            raw_path.into()
        }
    } else {
        raw_path.into()
    };

    // 3. Resolve path (Strict: must exist)
    let target_dir = match fs::canonicalize(&target_str) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("cd: {}: No such file or directory", target_str);
            return;
        }
    };
    // 4. Directory Check
    if !target_dir.is_dir() {
        eprintln!("cd: {}: Not a directory", target_str);
        return;
    }
    // 5. Execute
    if let Err(e) = env::set_current_dir(target_dir) {
        eprintln!("cd: {}: {}", target_str, e);
    }
}
