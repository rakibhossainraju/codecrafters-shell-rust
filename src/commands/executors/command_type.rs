use crate::commands::BuiltinCommands;
use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;
use crate::utils::print_command_not_found;

/// Execute the type builtin command
/// Shows information about a command (builtin or external)
pub fn execute_type(parsed_cmd: &ParsedCommand) {
    if parsed_cmd.args.is_empty() {
        eprintln!("type: missing argument");
        return;
    }

    for arg in parsed_cmd.args.iter() {
        if BuiltinCommands::is_builtin_command(arg) {
            println!("{} is a shell builtin", arg);
        } else {
            if let Some(path) = get_executable_path(arg) {
                println!("{} is {}", arg, path.display());
            } else {
                print_command_not_found(arg);
            }
        }
    }
}
