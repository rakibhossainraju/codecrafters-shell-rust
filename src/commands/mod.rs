mod builtin;
mod executors;
mod external;

pub use builtin::BuiltinCommands;
pub use external::ExternalCommand;
use crate::parser::ParsedCommand;

pub enum Command {
    Builtin(BuiltinCommands),
    External,
}

impl Command {
    pub fn execute(&self, parsed_cmd: ParsedCommand) {
        match self {
            Command::Builtin(BuiltinCommands::Exit) => {
                // Exit is handled in main loop, but included for completeness
            }
            Command::Builtin(BuiltinCommands::Cd) => {
                executors::cd::execute_cd(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Clear) => {
                executors::clear::execute_clear();
            }
            Command::Builtin(BuiltinCommands::Echo) => {
                executors::echo::execute_echo(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Help) => {
                executors::help::execute_help();
            }
            Command::Builtin(BuiltinCommands::Type) => {
                executors::command_type::execute_type(parsed_cmd);
            }
            Command::Builtin(BuiltinCommands::Pwd) => {
                executors::pwd::execute_pwd();
            }
            // External commands
            Command::External => {
                executors::external::execute_external_command(parsed_cmd);
            }
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match BuiltinCommands::from_str(s) {
            Some(builtin) => Command::Builtin(builtin),
            None => Command::External,
        }
    }
}
