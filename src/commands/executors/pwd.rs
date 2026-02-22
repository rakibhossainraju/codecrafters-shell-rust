use crate::error::{Result, ShellError};

pub fn execute_pwd() -> Result<()> {
    let path = std::env::current_dir().map_err(ShellError::IoError)?;
    println!("{}", path.display());
    Ok(())
}
