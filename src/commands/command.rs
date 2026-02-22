use crate::commands::{executors, BuiltinCommands, ExternalCommand};
use crate::parser::ParsedCommand;
use crate::error::{ShellError, Result};

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

    pub fn execute(&self) -> Result<()> {
        match self {
            Command::Builtin(BuiltinCommands::Exit, _) => {
                // Exit is handled in main loop, but included for completeness
                Ok(())
            }
            Command::Builtin(BuiltinCommands::Cd, parsed_cmd) => {
                executors::cd::execute_cd(parsed_cmd)
            }
            Command::Builtin(BuiltinCommands::Clear, _) => {
                executors::clear::execute_clear()
            }
            Command::Builtin(BuiltinCommands::Echo, parsed_cmd) => {
                executors::echo::execute_echo(parsed_cmd)
            }
            Command::Builtin(BuiltinCommands::Help, _) => {
                executors::help::execute_help()
            }
            Command::Builtin(BuiltinCommands::Type, parsed_cmd) => {
                executors::command_type::execute_type(parsed_cmd)
            }
            Command::Builtin(BuiltinCommands::Pwd, _) => {
                executors::pwd::execute_pwd()
            }
            // External commands
            Command::External(external_cmd) => {
                executors::external::execute_external_command(external_cmd)
            }
        }
    }
}
