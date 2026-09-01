use crate::error::{Result, ShellError};
use std::io::{Read, Write};

pub fn execute_jobs(_stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<()> {
    writeln!(stdout, "Jobs are not implemented yet")?;
    Ok(())
}
