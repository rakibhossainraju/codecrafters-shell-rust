mod builtin;
mod external;
mod executors;

pub use builtin::BuiltinCommands;


pub enum Command {
    Builtin(BuiltinCommands),
    External(String),
}

impl Command {
    pub fn execute(&self, args: &[String]) {
        match self {
            Command::Builtin(BuiltinCommands::Exit) => {
                // Exit is handled in main loop, but included for completeness
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
            Command::External(name) => {
                external::execute_external(name, args);
            }
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match BuiltinCommands::from_str(s) {
            Some(builtin) => Command::Builtin(builtin),
            None => Command::External(s.to_string()),
        }
    }
}

