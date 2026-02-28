use crate::error::{Result, ShellError};
use crate::state::ShellState;
use std::io::{Read, Write};

pub fn execute_history(
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    state: &mut ShellState,
) -> Result<()> {
    for (i, cmd) in state.history.iter().enumerate() {
        // {:>5} means "right-align this number padded to 5 spaces"
        // Then we add two literal spaces, then the command string.
        writeln!(stdout, "{:>5}  {}", i + 1, cmd).map_err(ShellError::IoError)?;
    }
    Ok(())
}
