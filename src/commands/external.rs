use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;
use std::path::PathBuf;

/// Represents an external command found in the system PATH
#[derive(Debug)]
pub struct ExternalCommand {
    pub path: PathBuf,
    pub ast: ParsedCommand,
}

impl ExternalCommand {
    /// Attempts to resolve a command by searching the system PATH
    /// Returns Some(ExternalCommand) if found, None otherwise
    pub fn try_resolve(ast: ParsedCommand) -> Option<Self> {
        let path = get_executable_path(&ast.cmd)?;
        Some(Self {
            path,
            ast,
        })
    }
}

/// Alternative: Implement TryFrom for ergonomic conversion
impl TryFrom<ParsedCommand> for ExternalCommand {
    type Error = ();

    fn try_from(ast: ParsedCommand) -> Result<Self, Self::Error> {
        Self::try_resolve(ast).ok_or(())
    }
}
