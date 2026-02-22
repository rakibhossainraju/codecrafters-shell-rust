use std::io::Write;
use crate::error::Result;

/// Execute the help builtin command
pub fn execute_help(_: &mut dyn Write) -> Result<()> {
    println!("Available builtin commands:");
    println!("  exit     - Exit the shell");
    println!("  echo     - Print text to stdout");
    println!("  type     - Show information about a command");
    println!("  help     - Show this help message");
    println!("  clear    - Clears screen clearing memory");
    Ok(())
}
