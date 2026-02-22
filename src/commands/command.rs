use crate::commands::{executors, BuiltinCommands, ExternalCommand};
use crate::parser::ParsedCommand;

pub enum Command {
    Builtin(BuiltinCommands, ParsedCommand),
    External(ExternalCommand),
}

impl Command {
    pub fn resolve(parsed_cmd: ParsedCommand) -> Option<Self> {
        // Try builtin commands first
        if let Some(builtin) = BuiltinCommands::from_str(&parsed_cmd.cmd) {
            return Some(Command::Builtin(builtin, parsed_cmd));
        }
        // Try external commands (searches PATH)
        ExternalCommand::try_resolve(parsed_cmd).map(Command::External)
    }

    pub fn execute(&self) {
        match self {
            Command::Builtin(BuiltinCommands::Exit, _) => {
                // Exit is handled in main loop, but included for completeness
            }
            Command::Builtin(BuiltinCommands::Cd, parsed_cmd) => {
                executors::cd::execute_cd(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Clear, _) => {
                executors::clear::execute_clear();
            }
            Command::Builtin(BuiltinCommands::Echo, parsed_cmd) => {
                executors::echo::execute_echo(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Help, _) => {
                executors::help::execute_help();
            }
            Command::Builtin(BuiltinCommands::Type, parsed_cmd) => {
                executors::command_type::execute_type(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Pwd, _) => {
                executors::pwd::execute_pwd();
            }
            // External commands
            Command::External(external_cmd) => {
                executors::external::execute_external_command(external_cmd);
            }
        }
    }
}
