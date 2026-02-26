use crate::commands::ExternalCommand;
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::process::{Child, ChildStdout, Stdio};

pub fn execute_pipeline(cmds: Vec<ParsedCommand>) -> Result<()> {
    let mut previous_stdout: Option<ChildStdout> = None;
    let mut children: Vec<Child> = Vec::new();
    let cmd_count = cmds.len();

    for (i, parsed_cmd) in cmds.into_iter().enumerate() {
        let external_cmd = ExternalCommand::try_from(parsed_cmd)?;

        let stdin = previous_stdout.take().map(Stdio::from);
        let is_last = i == cmd_count - 1;
        let stdout = if !is_last { Some(Stdio::piped()) } else { None };

        let mut child = external_cmd.spawn(stdin, stdout)?;

        if !is_last {
            previous_stdout = child.stdout.take();
        }
        children.push(child);
    }

    for mut child in children {
        child.wait().map_err(|_| {
            ShellError::WaitError(format!("pipeline. child-id {}", child.id()))
        })?;
    }
    Ok(())
}
