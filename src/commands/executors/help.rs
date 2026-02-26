use crate::error::Result;
use std::io::{Read, Write};

/// Execute the help builtin command
pub fn execute_help(_stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<()> {
    writeln!(stdout, "Available builtin commands:")?;
    writeln!(stdout, "  exit     - Exit the shell")?;
    writeln!(stdout, "  echo     - Print text to stdout")?;
    writeln!(stdout, "  type     - Show information about a command")?;
    writeln!(stdout, "  help     - Show this help message")?;
    writeln!(stdout, "  clear    - Clears screen clearing memory")?;
    Ok(())
}
