use std::fs::File;
use crate::commands::ExternalCommand;
use crate::error::{Result, ShellError};
use std::os::unix::prelude::CommandExt;
use std::process::{Command as OsCommand, Stdio};
use crate::parser::{Descriptor, RedirectionType};

pub fn execute_external_command(external_cmd: &ExternalCommand) -> Result<()> {
    // The path is already resolved in ExternalCommand
    let mut cmd = OsCommand::new(&external_cmd.path);

    cmd.arg0(&external_cmd.ast.cmd);
    cmd.args(&external_cmd.ast.args);

    for redirect in &external_cmd.ast.redirects {
        if redirect.redirection_type == RedirectionType::Output && redirect.descriptor == Descriptor::Stdout {
            match File::create(&redirect.file) {
                Ok(file) => {
                    // Tell the OS to route this child process's stdout into the file!
                    cmd.stdout(Stdio::from(file));
                }
                Err(e) => {
                    eprintln!("shell: {}: {}", redirect.file, &e);
                    return Err(ShellError::IoError(e)); // Abort if we can't open the file
                }
            }
        }
    }

    let mut child = cmd.spawn().map_err(|error| ShellError::ExecutionError {
        command: external_cmd.ast.cmd.clone(),
        source: error,
    })?;

    let status = child
        .wait()
        .map_err(|_| ShellError::WaitError(external_cmd.ast.cmd.clone()))?;

    // if !status.success() {
    //     return Err(ShellError::ExitWithStatus {
    //         command: external_cmd.ast.cmd.clone(),
    //         status,
    //     });
    // }

    Ok(())
}
