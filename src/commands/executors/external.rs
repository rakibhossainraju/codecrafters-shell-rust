use crate::commands::ExternalCommand;
use crate::error::{Result, ShellError};
use crate::utils::redirection::ResolvedRedirections;
use std::os::unix::prelude::CommandExt;
use std::process::{Command as OsCommand, Stdio};

pub fn execute_external_command(external_cmd: &ExternalCommand) -> Result<()> {
    // The path is already resolved in ExternalCommand
    let mut cmd = OsCommand::new(&external_cmd.path);

    cmd.arg0(&external_cmd.ast.cmd);
    cmd.args(&external_cmd.ast.args);

    let resolved = ResolvedRedirections::resolve(&external_cmd.ast)?;

    if let Some(stdout) = resolved.stdout {
        cmd.stdout(Stdio::from(stdout));
    }
    if let Some(stderr) = resolved.stderr {
        cmd.stderr(Stdio::from(stderr));
    }
    if let Some(stdin) = resolved.stdin {
        cmd.stdin(Stdio::from(stdin));
    }

    let mut child = cmd.spawn().map_err(|error| ShellError::ExecutionError {
        command: external_cmd.ast.cmd.clone(),
        source: error,
    })?;

    child
        .wait()
        .map_err(|_| ShellError::WaitError(external_cmd.ast.cmd.clone()))?;

    Ok(())
}
