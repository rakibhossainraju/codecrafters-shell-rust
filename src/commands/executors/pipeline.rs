use crate::commands::ExternalCommand;
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::process::{Child, ChildStdout, Command as OsCommand, Stdio};

pub fn execute_pipeline(cmds: Vec<ParsedCommand>) -> Result<()> {
    let mut previous_stdout: Option<ChildStdout> = None;
    let mut children: Vec<Child> = Vec::new();
    let cmd_count = cmds.len();
    for (i, parsed_cmd) in cmds.into_iter().enumerate() {
        let external_cmd = match ExternalCommand::try_resolve(parsed_cmd.clone()) {
            Some(cmd) => cmd,
            None => return Err(ShellError::CommandNotFound(parsed_cmd.cmd)),
        };
        let mut os_cmd = OsCommand::new(&external_cmd.path);

        if let Some(stdout) = previous_stdout.take() {
            os_cmd.stdin(Stdio::from(stdout));
        }
        let is_last = i == cmd_count - 1;
        if !is_last {
            os_cmd.stdout(Stdio::piped());
        }

        let mut child = os_cmd.spawn().map_err(|err| ShellError::ExecutionError {
            command: external_cmd.ast.cmd.clone(),
            source: err,
        })?;

        if !is_last {
            previous_stdout = child.stdout.take();
        }
        children.push(child);
    }
    for mut child in children {
        child.wait().map_err(|_| {
            ShellError::WaitError(format!("pipeline. child-id {}", child.id().to_string()))
        })?;
    }
    Ok(())
}
