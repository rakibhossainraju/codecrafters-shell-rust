use crate::error::{Result, ShellError};
use std::io::{Read, Write};

pub fn execute_pwd(_stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<()> {
    let path = std::env::current_dir().map_err(ShellError::IoError)?;
    writeln!(stdout, "{}", path.display()).map_err(ShellError::IoError)?;
    Ok(())
}
