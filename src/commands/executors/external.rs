use crate::commands::ExternalCommand;
use crate::error::{Result, ShellError};

pub fn execute_external_command(external_cmd: &ExternalCommand) -> Result<()> {
    let mut child = external_cmd.spawn(None, None)?;

    child
        .wait()
        .map_err(|_| ShellError::WaitError(external_cmd.parsed_cmd.cmd.clone()))?;

    Ok(())
}
