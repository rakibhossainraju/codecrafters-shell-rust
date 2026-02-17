use crate::utils::path::get_executable_path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ExternalCommand {
    pub name: String,
    pub path: Option<PathBuf>, // Full path to the executable if found
    pub args: Option<Vec<String>>,
}

impl ExternalCommand {
    /// Creates a new ExternalCommandBuilder with the given name
    pub fn new(name: impl Into<String>, args: Option<Vec<String>>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            path: get_executable_path(&name),
            args,
        }
    }

    pub fn path_to_string(&self) -> Option<String> {
        self.path.as_ref().map(|p| p.to_string_lossy().to_string())
    }
}
