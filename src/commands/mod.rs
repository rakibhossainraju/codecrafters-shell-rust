mod builtin;
mod executors;
mod external;

pub use builtin::BuiltinCommands;
pub use external::ExternalCommand;

pub enum Command {
    Builtin(BuiltinCommands),
    External,
}

impl Command {
    pub fn execute(&self, user_input: Vec<String>) {
        let cmd_name = user_input[0].clone();
        let args = &user_input[1..];
        match self {
            Command::Builtin(BuiltinCommands::Exit) => {
                // Exit is handled in main loop, but included for completeness
            }
            Command::Builtin(BuiltinCommands::Cd) => {
                executors::cd::execute_cd(args);
            }
            Command::Builtin(BuiltinCommands::Clear) => {
                executors::clear::execute_clear();
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
            Command::Builtin(BuiltinCommands::Pwd) => {
                executors::pwd::execute_pwd();
            }
            // External commands
            Command::External => {
                executors::external::execute_external_command(ExternalCommand::new(
                    cmd_name,
                    Some(args.to_vec()),
                ));
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
