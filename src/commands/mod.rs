mod builtin;
mod executors;
mod external;

pub use builtin::BuiltinCommands;
pub use external::ExternalCommand;

pub enum Command {
    Builtin(BuiltinCommands),
    External(ExternalCommand),
}

impl Command {
    pub fn execute(&self, args: &[String]) {
        match self {
            Command::Builtin(BuiltinCommands::Exit) => {
                // Exit is handled in main loop, but included for completeness
            }
            Command::Builtin(BuiltinCommands::Clear) => {
                executors::clear::execute_clear();
            }
            Command::External(ext) => {
                executors::external::execute_external_command(ext);
            }
            Command::Builtin(BuiltinCommands::Echo) => {
                executors::echo::execute_echo(args);
            }
            Command::Builtin(BuiltinCommands::Help) => {
                executors::help::execute_help();
            }
            Command::Builtin(BuiltinCommands::Type) => {
                executors::command_type::execute_type(args);
            }
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match BuiltinCommands::from_str(s) {
            Some(builtin) => Command::Builtin(builtin),
            None => Command::External(ExternalCommand::new(s.to_string())),
        }
    }
}
