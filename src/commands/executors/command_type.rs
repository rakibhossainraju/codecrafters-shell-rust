use crate::commands::{BuiltinCommands, ExternalCommand};
use crate::utils::print_command_not_found;

/// Execute the type builtin command
/// Shows information about a command (builtin or external)
pub fn execute_type(args: &[String]) {
    if args.is_empty() {
        eprintln!("type: missing argument");
        return;
    }

    for arg in args {
        if BuiltinCommands::is_builtin_command(arg) {
            println!("{} is a shell builtin", arg);
        } else {
            if let Some(cmd) = ExternalCommand::resolve(arg) {
                println!("{} is {}", cmd.name, cmd.path.unwrap());
            } else {
                print_command_not_found(arg)
            }
        }
    }
}
