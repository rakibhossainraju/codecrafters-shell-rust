use crate::utils::get_os_paths;

pub struct ExternalCommand {
    pub name: String,
    pub path: Option<String>, // Full path to the executable if found
}

impl ExternalCommand {
    pub fn new(name: String) -> Self {
        ExternalCommand { name, path: None }
    }

    pub fn with_path(name: String, path: String) -> Self {
        ExternalCommand {
            name,
            path: Some(path),
        }
    }

    /// Resolves the external command by searching in PATH.
    /// Returns Some(ExternalCommand) with the full path if found, None otherwise.
    pub fn resolve(name: &str) -> Option<Self> {
        let name = name.to_string();
        if let Some(paths) = get_os_paths() {
            for path_str in paths {
                let full_path = path_str.join(&name);
                if full_path.exists() && full_path.is_file() {
                    return Some(ExternalCommand::with_path(
                        name,
                        full_path.to_string_lossy().to_string(),
                    ));
                }
            }
        }
        None
    }
}
