use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::redirection::ResolvedRedirections;
use std::process::{Command, Stdio};

pub fn execute_clear(parsed_cmd: &ParsedCommand) -> Result<()> {
    // Clear the terminal screen
    // This is a simple implementation that works on Unix-like systems and Windows
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(&["/C", "cls"]);
        c
    } else {
        Command::new("clear")
    };

    let resolved = ResolvedRedirections::resolve(parsed_cmd)?;
    if let Some(stdout) = resolved.stdout {
        cmd.stdout(Stdio::from(stdout));
    }
    if let Some(stderr) = resolved.stderr {
        cmd.stderr(Stdio::from(stderr));
    }

    let status = cmd.status().map_err(ShellError::IoError)?;

    if !status.success() {
        return Err(ShellError::ExitWithStatus {
            command: "clear".to_string(),
            status,
        });
    }

    Ok(())
}
