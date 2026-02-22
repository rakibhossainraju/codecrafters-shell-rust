use crate::error::{Result, ShellError};
use std::process::Command;

pub fn execute_clear() -> Result<()> {
    // Clear the terminal screen
    // This is a simple implementation that works on Unix-like systems and Windows
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .map_err(ShellError::IoError)?
    } else {
        Command::new("clear")
            .status()
            .map_err(ShellError::IoError)?
    };

    if !status.success() {
        return Err(ShellError::ExitWithStatus {
            command: "clear".to_string(),
            status,
        });
    }

    Ok(())
}
