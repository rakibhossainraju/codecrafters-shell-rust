use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;
use crate::utils::redirection::ResolvedRedirections;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command as OsCommand, Stdio};

/// Represents an external command found in the system PATH
#[derive(Debug)]
pub struct ExternalCommand {
    pub path: PathBuf,
    pub parsed_cmd: ParsedCommand,
}

impl ExternalCommand {
    /// Attempts to resolve a command by searching the system PATH
    /// Returns Some(ExternalCommand) if found, None otherwise
    pub fn try_resolve(parsed: ParsedCommand) -> Option<Self> {
        let path = get_executable_path(&parsed.cmd)?;
        Some(Self {
            path,
            parsed_cmd: parsed,
        })
    }

    pub fn spawn(&self, stdin: Option<Stdio>, stdout: Option<Stdio>) -> Result<Child> {
        let mut cmd = self.as_os_command(stdin, stdout)?;
        cmd.spawn().map_err(|error| ShellError::ExecutionError {
            command: self.parsed_cmd.cmd.clone(),
            source: error,
        })
    }

    pub fn as_os_command(
        &self,
        default_stdin: Option<Stdio>,
        default_stdout: Option<Stdio>,
    ) -> Result<OsCommand> {
        let mut cmd = OsCommand::new(&self.path);

        // Set argv[0] to the command name (e.g., "ls" instead of "/bin/ls")
        cmd.arg0(&self.parsed_cmd.cmd);
        cmd.args(&self.parsed_cmd.args);

        let resolved = ResolvedRedirections::resolve(&self.parsed_cmd)?;

        // Redirections take precedence over default pipeline streams
        if let Some(stdin) = resolved.stdin {
            cmd.stdin(Stdio::from(stdin));
        } else if let Some(stdin) = default_stdin {
            cmd.stdin(stdin);
        }

        if let Some(stdout) = resolved.stdout {
            cmd.stdout(Stdio::from(stdout));
        } else if let Some(stdout) = default_stdout {
            cmd.stdout(stdout);
        }

        if let Some(stderr) = resolved.stderr {
            cmd.stderr(Stdio::from(stderr));
        }

        Ok(cmd)
    }
}

/// Alternative: Implement TryFrom for ergonomic conversion
impl TryFrom<ParsedCommand> for ExternalCommand {
    type Error = ShellError;

    fn try_from(parsed: ParsedCommand) -> Result<Self> {
        let cmd_name = parsed.cmd.clone();
        Self::try_resolve(parsed).ok_or_else(|| ShellError::CommandNotFound(cmd_name))
    }
}
