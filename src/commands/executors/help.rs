use crate::commands::BuiltinCommands;
use crate::error::Result;
use std::io::{Read, Write};

use strum::IntoEnumIterator;

/// Get the help text for a builtin command
fn get_command_help(cmd: BuiltinCommands) -> &'static str {
    match cmd {
        BuiltinCommands::Exit => "exit     - Exit the shell",
        BuiltinCommands::Echo => "echo     - Print text to stdout",
        BuiltinCommands::Help => "help     - Show this help message",
        BuiltinCommands::Type => "type     - Show information about a command",
        BuiltinCommands::Pwd => "pwd      - Print working directory",
        BuiltinCommands::Cd => "cd       - Change directory",
        BuiltinCommands::Clear => "clear    - Clear the screen",
        BuiltinCommands::History => "history  - Show command history",
        BuiltinCommands::Jobs => "jobs     - Show running jobs",
    }
}

/// Execute the help builtin command
pub fn execute_help(_stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<()> {
    writeln!(stdout, "Available builtin commands:")?;

    for cmd in BuiltinCommands::iter() {
        writeln!(stdout, "  {}", get_command_help(cmd))?;
    }

    Ok(())
}
