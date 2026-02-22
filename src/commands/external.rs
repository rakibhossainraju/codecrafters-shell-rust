use crate::parser::ParsedCommand;
use crate::utils::path::get_executable_path;
use std::path::PathBuf;

/// Represents an external command found in the system PATH
#[derive(Debug)]
pub struct ExternalCommand {
    pub name: String,
    pub path: PathBuf,
    pub ast: ParsedCommand,
}

impl ExternalCommand {
    /// Attempts to resolve a command by searching the system PATH
    /// Returns Some(ExternalCommand) if found, None otherwise
    pub fn try_resolve(ast: ParsedCommand) -> Option<Self> {
        let path = get_executable_path(&ast.cmd)?;
        Some(Self {
            name: ast.cmd.clone(),
            path,
            ast,
        })
    }

    /// Checks if a command exists in PATH without creating the struct
    pub fn exists(cmd_name: &str) -> bool {
        get_executable_path(cmd_name).is_some()
    }

    /// Gets the path as a string, returning None if path contains invalid UTF-8
    pub fn path_str(&self) -> Option<&str> {
        self.path.to_str()
    }

    /// Gets the path as a string, using lossy conversion for invalid UTF-8
    pub fn path_string_lossy(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

/// Alternative: Implement TryFrom for ergonomic conversion
impl TryFrom<ParsedCommand> for ExternalCommand {
    type Error = ();

    fn try_from(ast: ParsedCommand) -> Result<Self, Self::Error> {
        Self::try_resolve(ast).ok_or(())
    }
}
