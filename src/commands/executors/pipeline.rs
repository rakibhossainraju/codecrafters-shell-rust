use std::io::{self, Read, Write};
use crate::commands::{BuiltinCommands, Command, ExternalCommand};
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::process::{Child, ChildStdout, Stdio};
use std::mem;

pub enum PipelineInput {
    None,
    ChildStdout(ChildStdout),
    Buffer(Vec<u8>),
}

impl Default for PipelineInput {
    fn default() -> Self {
        PipelineInput::None
    }
}

pub fn execute_pipeline(cmds: Vec<ParsedCommand>) -> Result<()> {
    let mut previous_stdout = PipelineInput::None;
    let mut children: Vec<Child> = Vec::new();
    let cmd_count = cmds.len();

    for (i, parsed_cmd) in cmds.into_iter().enumerate() {
        let is_last = i == cmd_count - 1;

        // Try to resolve the command
        let cmd = Command::resolve(parsed_cmd)?;
        
        match cmd {
            Command::Builtin(_, _) => {
                if !is_last {
                    // Capture output to buffer for piping to next command
                    let mut buffer = Vec::new();
                    {
                        // Scope the borrow so it ends before we move buffer
                        cmd.execute(Some(Box::new(&mut buffer)))?;
                    }
                    previous_stdout = PipelineInput::Buffer(buffer);
                } else {
                    // Last command: write directly to stdout
                    cmd.execute(Some(Box::new(io::stdout())))?;
                    previous_stdout = PipelineInput::None;
                }
            }
            Command::External(external_cmd) => {
                let stdin = match &previous_stdout {
                    PipelineInput::None => None,
                    PipelineInput::Buffer(_) => Some(Stdio::piped()),
                    PipelineInput::ChildStdout(_) => {
                        // We have to extract it out of the enum to take ownership
                        if let PipelineInput::ChildStdout(stdout) = mem::take(&mut previous_stdout) {
                            Some(Stdio::from(stdout))
                        } else {
                            unreachable!()
                        }
                    }
                };

                let stdout = if !is_last { Some(Stdio::piped()) } else { None };
                let mut child = external_cmd.spawn(stdin, stdout)?;

                if let PipelineInput::Buffer(buff) = &previous_stdout {
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(buff)?;
                    }
                }

                if !is_last {
                    previous_stdout = PipelineInput::ChildStdout(child.stdout.take().unwrap());
                }
                children.push(child);
            }
        }
    }

    for mut child in children {
        child.wait().map_err(|_| {
            ShellError::WaitError(format!("pipeline. child-id {}", child.id()))
        })?;
    }
    Ok(())
}
