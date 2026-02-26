use std::io::{self, Read, Write, Cursor};
use crate::commands::{Command, ExternalCommand};
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::process::{Child, ChildStdout, Stdio};
use std::mem;

/// Represents the output of a command that will be used as the input for the next command in the pipeline
pub enum PipelineLink {
    None,
    ChildStdout(ChildStdout),
    Buffer(Vec<u8>),
}

impl Default for PipelineLink {
    fn default() -> Self {
        PipelineLink::None
    }
}

impl PipelineLink {
    /// Converts the link into a reader that can be used by built-in commands
    pub fn into_reader(self) -> Option<Box<dyn Read>> {
        match self {
            PipelineLink::None => None,
            PipelineLink::ChildStdout(stdout) => Some(Box::new(stdout)),
            PipelineLink::Buffer(buf) => Some(Box::new(Cursor::new(buf))),
        }
    }
}

pub struct Pipeline {
    commands: Vec<ParsedCommand>,
    children: Vec<Child>,
    previous_link: PipelineLink,
}

impl Pipeline {
    pub fn new(commands: Vec<ParsedCommand>) -> Self {
        Self {
            commands,
            children: Vec::new(),
            previous_link: PipelineLink::None,
        }
    }

    pub fn run(mut self) -> Result<()> {
        let cmd_count = self.commands.len();
        // Take ownership of the commands so we can iterate over them
        let commands = mem::take(&mut self.commands);

        for (i, parsed_cmd) in commands.into_iter().enumerate() {
            let is_last = i == cmd_count - 1;
            let cmd = Command::resolve(parsed_cmd)?;

            match cmd {
                Command::Builtin(_, _) => self.execute_builtin(cmd, is_last)?,
                Command::External(external_cmd) => self.execute_external(external_cmd, is_last)?,
            }
        }

        self.wait_for_children()
    }

    fn execute_builtin(&mut self, cmd: Command, is_last: bool) -> Result<()> {
        let stdin = mem::take(&mut self.previous_link).into_reader();
        
        if !is_last {
            // Builtin commands don't have a stdout we can pipe directly, 
            // so we capture their output in a buffer for the next command.
            let mut buffer = Vec::new();
            cmd.execute(stdin, Some(Box::new(&mut buffer)))?;
            self.previous_link = PipelineLink::Buffer(buffer);
        } else {
            // Last command: write directly to stdout
            cmd.execute(stdin, Some(Box::new(io::stdout())))?;
            self.previous_link = PipelineLink::None;
        }
        Ok(())
    }

    fn execute_external(&mut self, external_cmd: ExternalCommand, is_last: bool) -> Result<()> {
        let stdin_config = match &self.previous_link {
            PipelineLink::None => None,
            PipelineLink::Buffer(_) => Some(Stdio::piped()),
            PipelineLink::ChildStdout(_) => {
                if let PipelineLink::ChildStdout(stdout) = mem::take(&mut self.previous_link) {
                    Some(Stdio::from(stdout))
                } else {
                    unreachable!()
                }
            }
        };

        let stdout_config = if !is_last { Some(Stdio::piped()) } else { None };
        let mut child = external_cmd.spawn(stdin_config, stdout_config)?;

        // If the previous command was a builtin, we need to manually write its captured output to the new child's stdin
        if let PipelineLink::Buffer(buff) = mem::take(&mut self.previous_link) {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&buff)?;
            }
        }

        if !is_last {
            self.previous_link = PipelineLink::ChildStdout(child.stdout.take().unwrap());
        } else {
            self.previous_link = PipelineLink::None;
        }

        self.children.push(child);
        Ok(())
    }

    fn wait_for_children(&mut self) -> Result<()> {
        for mut child in mem::take(&mut self.children) {
            child.wait().map_err(|_| {
                ShellError::WaitError(format!("pipeline child-id {}", child.id()))
            })?;
        }
        Ok(())
    }
}

pub fn execute_pipeline(cmds: Vec<ParsedCommand>) -> Result<()> {
    Pipeline::new(cmds).run()
}
