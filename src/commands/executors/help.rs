use crate::error::Result;
use std::io::{Read, Write};
use crate::commands::BuiltinCommands;

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
    }
}

/// Execute the help builtin command
pub fn execute_help(_stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<()> {
    writeln!(stdout, "Available builtin commands:")?;

    // This array ensures all builtin commands are listed
    let commands = [
        BuiltinCommands::Exit,
        BuiltinCommands::Echo,
        BuiltinCommands::Help,
        BuiltinCommands::Type,
        BuiltinCommands::Pwd,
        BuiltinCommands::Cd,
        BuiltinCommands::Clear,
        BuiltinCommands::History,
    ];

    for cmd in commands.iter() {
        writeln!(stdout, "  {}", get_command_help(*cmd))?;
    }

    Ok(())
}
