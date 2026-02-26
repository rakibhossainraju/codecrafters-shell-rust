use std::io::{Read, Write};
use crate::commands::{
    BuiltinCommands, ExternalCommand,
    executors::{cd, clear, command_type, echo, external, help, pwd},
};
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::redirection::{IoStreams, ResolvedRedirections};

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
    ) -> Result<()> {
        match self {
            Command::Builtin(builtin, parsed_cmd) => {
                let resolved = ResolvedRedirections::resolve(parsed_cmd)?;
                let io_streams = IoStreams::from_resolved(resolved);

                let mut stdin = if let Some(in_stream) = input {
                    in_stream
                } else {
                    io_streams.stdin
                };

                let mut stdout = if let Some(out_stream) = output {
                    out_stream
                } else {
                    io_streams.stdout
                };

                match builtin {
                    BuiltinCommands::Cd => cd::execute_cd(parsed_cmd, &mut stdin, &mut stdout),
                    BuiltinCommands::Clear => clear::execute_clear(parsed_cmd),
                    BuiltinCommands::Echo => echo::execute_echo(parsed_cmd, &mut stdin, &mut stdout),
                    BuiltinCommands::Help => help::execute_help(&mut stdin, &mut stdout),
                    BuiltinCommands::Type => {
                        command_type::execute_type(parsed_cmd, &mut stdin, &mut stdout)
                    }
                    BuiltinCommands::Pwd => pwd::execute_pwd(&mut stdin, &mut stdout),
                    BuiltinCommands::Exit => Ok(()),
                }
            }
            // External commands
            Command::External(external_cmd) => external::execute_external_command(external_cmd),
        }
    }
}
