use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::io::Write;
use std::{env, fs};

pub fn execute_cd(parsed_cmd: &ParsedCommand, _: &mut dyn Write) -> Result<()> {
    // 1. Get the raw path string, defaulting to "~" if empty
    let raw_path = parsed_cmd.args.first().map(|s| s.as_str()).unwrap_or("~");

    // 2. Handle Home Directory Expansion.
    // We must handle specific cases: "~" alone, or paths starting with "~/"
    let target_str = if raw_path == "~" {
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
    let target_dir =
        fs::canonicalize(&target_str).map_err(|_| ShellError::CdError(target_str.clone()))?;

    // 4. Directory Check
    if !target_dir.is_dir() {
        return Err(ShellError::CdError(target_str.clone()));
    }
    // 5. Execute
    env::set_current_dir(target_dir).map_err(|_| ShellError::CdError(target_str.clone()))?;

    Ok(())
}
