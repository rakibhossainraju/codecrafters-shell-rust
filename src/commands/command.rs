use crate::commands::executors::history;
use crate::commands::{
    BuiltinCommands, ExternalCommand,
    executors::{cd, clear, command_type, echo, external, help, pwd},
};
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::state::ShellState;
use crate::utils::redirection::{IoStreams, ResolvedRedirections};
use std::io::{Read, Write};

pub enum Command {
    Builtin(BuiltinCommands, ParsedCommand),
    External(ExternalCommand),
}

impl Command {
    pub fn resolve(parsed_cmd: ParsedCommand) -> Result<Self> {
        if parsed_cmd.cmd.is_empty() {
            return Err(ShellError::SyntaxError("empty command".to_string()));
        }

        // Try builtin commands first
        if let Some(builtin) = BuiltinCommands::from_str(&parsed_cmd.cmd) {
            return Ok(Command::Builtin(builtin, parsed_cmd));
        }
        // Try external commands (searches PATH)
        let cmd_name = parsed_cmd.cmd.clone();
        ExternalCommand::try_resolve(parsed_cmd)
            .map(Command::External)
            .ok_or_else(|| ShellError::CommandNotFound(cmd_name))
    }

    pub fn execute<'a>(
        &self,
        input: Option<Box<dyn Read + 'a>>,
        output: Option<Box<dyn Write + 'a>>,
        state: &mut ShellState,
    ) -> Result<()> {
        match self {
            Command::Builtin(builtin, parsed_cmd) => {
                let resolved = ResolvedRedirections::resolve(parsed_cmd)?;

                let mut stdin: Box<dyn Read> = match resolved.stdin {
                    Some(file) => Box::new(file),
                    None => input.unwrap_or_else(|| Box::new(std::io::stdin())),
                };
                let mut stdout: Box<dyn Write> = match resolved.stdout {
                    Some(file) => Box::new(file),
                    None => output.unwrap_or_else(|| Box::new(std::io::stdout())),
                };

                match builtin {
                    BuiltinCommands::Cd => cd::execute_cd(parsed_cmd, &mut stdin, &mut stdout),
                    BuiltinCommands::Clear => clear::execute_clear(parsed_cmd),
                    BuiltinCommands::Echo => {
                        echo::execute_echo(parsed_cmd, &mut stdin, &mut stdout)
                    }
                    BuiltinCommands::Help => help::execute_help(&mut stdin, &mut stdout),
                    BuiltinCommands::History => {
                        history::execute_history(&mut stdin, &mut stdout, state)
                    }
                    BuiltinCommands::Pwd => pwd::execute_pwd(&mut stdin, &mut stdout),
                    BuiltinCommands::Type => {
                        command_type::execute_type(parsed_cmd, &mut stdin, &mut stdout)
                    }
                    BuiltinCommands::Exit => Ok(()),
                }
            }
            // External commands
            Command::External(external_cmd) => external::execute_external_command(external_cmd),
        }
    }
}
