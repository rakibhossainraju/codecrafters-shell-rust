use crate::commands::BuiltinCommands;
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;

/// Execute the type builtin command
/// Shows information about a command (builtin or external)
pub fn execute_type(parsed_cmd: &ParsedCommand) -> Result<()> {
    if parsed_cmd.args.is_empty() {
        return Err(ShellError::SyntaxError(
            "type: missing argument".to_string(),
        ));
    }

    for arg in parsed_cmd.args.iter() {
        if BuiltinCommands::is_builtin_command(arg) {
            println!("{} is a shell builtin", arg);
        } else {
            if let Some(path) = get_executable_path(arg) {
                println!("{} is {}", arg, path.display());
            } else {
                return Err(ShellError::CommandNotFound(arg.clone()));
            }
        }
    }
    Ok(())
}
