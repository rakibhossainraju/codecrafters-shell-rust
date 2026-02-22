use crate::commands::ExternalCommand;
use crate::error::{ShellError, Result};
use std::os::unix::prelude::CommandExt;
use std::process::Command as OsCommand;

pub fn execute_external_command(external_cmd: &ExternalCommand) -> Result<()> {
    // The path is already resolved in ExternalCommand
    let mut cmd = OsCommand::new(&external_cmd.path);

    // Set arg0 to the command name (not full path)
    cmd.arg0(&external_cmd.ast.cmd);

    // Add all arguments
    cmd.args(&external_cmd.ast.args);

    // TODO:: Add the redirect logic!
    // if let Some(out_file) = external_cmd.ast.redirect_out { ... }

    let mut child = cmd.spawn().map_err(|error| ShellError::ExecutionError {
        command: external_cmd.ast.cmd.clone(),
        source: error,
    })?;

    let status = child.wait().map_err(|_| ShellError::WaitError(external_cmd.ast.cmd.clone()))?;

    if !status.success() {
        return Err(ShellError::ExitWithStatus {
            command: external_cmd.ast.cmd.clone(),
            status,
        });
    }

    Ok(())
}
