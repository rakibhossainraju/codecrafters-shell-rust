use crate::commands::BuiltinCommands;
use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;
use std::io::{Read, Write};

/// Execute the type builtin command
/// Shows information about a command (builtin or external)
pub fn execute_type(
    parsed_cmd: &ParsedCommand,
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<()> {
    if parsed_cmd.args.is_empty() {
        return Err(ShellError::SyntaxError(
            "type: missing argument".to_string(),
        ));
    }

    for arg in parsed_cmd.args.iter() {
        if BuiltinCommands::is_builtin_command(arg) {
            writeln!(stdout, "{} is a shell builtin", arg)?;
        } else {
            if let Some(path) = get_executable_path(arg) {
                writeln!(stdout, "{} is {}", arg, path.display())?;
            } else {
                return Err(ShellError::CommandNotFound(arg.clone()));
            }
        }
    }
    Ok(())
}
