use crate::commands::{
    executors::{cd, clear, command_type, echo, external, help, pwd},
    BuiltinCommands,
    ExternalCommand,
};
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;

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
                // Exit is handled in the main loop but included for completeness
                Ok(())
            }
            Command::Builtin(BuiltinCommands::Cd, parsed_cmd) => cd::execute_cd(parsed_cmd),
            Command::Builtin(BuiltinCommands::Clear, _) => clear::execute_clear(),
            Command::Builtin(BuiltinCommands::Echo, parsed_cmd) => echo::execute_echo(parsed_cmd),
            Command::Builtin(BuiltinCommands::Help, _) => help::execute_help(),
            Command::Builtin(BuiltinCommands::Type, parsed_cmd) => {
                command_type::execute_type(parsed_cmd)
            }
            Command::Builtin(BuiltinCommands::Pwd, _) => pwd::execute_pwd(),
            // External commands
            Command::External(external_cmd) => external::execute_external_command(external_cmd),
        }
    }
}
