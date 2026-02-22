use crate::commands::{executors, BuiltinCommands, ExternalCommand};
use crate::parser::ParsedCommand;
use std::convert::TryFrom;

pub enum Command {
    Builtin(BuiltinCommands, ParsedCommand),
    External(ExternalCommand),
}

impl Command {
    pub fn resolve(parsed_cmd: ParsedCommand) -> Option<Self> {
        // Try to resolve as builtin first (using as_str() to avoid moving)
        if let Ok(builtin) = BuiltinCommands::try_from(parsed_cmd.cmd.as_str()) {
            return Some(Command::Builtin(builtin, parsed_cmd));
        }

        // Try to resolve as external command
        ExternalCommand::try_from(parsed_cmd)
            .ok()
            .map(Command::External)
    }

    pub fn execute(&self) {
        match self {
            Command::Builtin(BuiltinCommands::Exit, _) => {
                // Exit is handled in main loop, but included for completeness
            }
            Command::Builtin(BuiltinCommands::Cd, parsed_cmd) => {
                executors::cd::execute_cd(parsed_cmd.clone());
            }
            Command::Builtin(BuiltinCommands::Clear, _) => {
                executors::clear::execute_clear();
            }
            Command::Builtin(BuiltinCommands::Echo, parsed_cmd) => {
                executors::echo::execute_echo(parsed_cmd.clone());
            }
            Command::Builtin(BuiltinCommands::Help, _) => {
                executors::help::execute_help();
            }
            Command::Builtin(BuiltinCommands::Type, parsed_cmd) => {
                executors::command_type::execute_type(parsed_cmd.clone());
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
