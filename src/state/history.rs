use crate::error::Result;
use crate::error::ShellError;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;
use std::io::{BufRead, BufReader};

pub struct HistoryState {
    pub entries: Vec<String>,
    pub last_appended_index: usize,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            last_appended_index: 0,
        }
    }
}

impl HistoryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_history(&mut self, cmd: String) {
        self.entries.push(cmd);
    }

    /// Loads history from `filename`. A missing file is expected on first run
    /// (nothing has been saved yet) and is treated as a successful no-op;
    /// any other I/O failure (permissions, etc.) is a real error.
    pub fn load_history(&mut self, filename: &str) -> Result<()> {
        let file = match File::open(filename) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                self.entries.push(trimmed.to_string());
            }
        }
        Ok(())
    }

    pub fn write_history(&self, filename: &str) -> Result<()> {
        use std::io::Write;
        let mut file = File::create(filename).map_err(ShellError::HistoryWriteError)?;
        for cmd in &self.entries {
            writeln!(file, "{}", cmd).map_err(ShellError::HistoryWriteError)?;
        }
        Ok(())
    }

    pub fn append_history(&mut self, filename: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .map_err(ShellError::HistoryWriteError)?;

        let start_index = self.last_appended_index;
        let end_index = self.entries.len();

        for i in start_index..end_index {
            writeln!(file, "{}", self.entries[i]).map_err(ShellError::HistoryWriteError)?;
        }

        self.last_appended_index = end_index;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn new_state_has_empty_history() {
        let state = HistoryState::new();
        assert!(state.entries.is_empty());
        assert_eq!(state.last_appended_index, 0);
    }

    #[test]
    fn add_history_pushes_a_single_entry() {
        let mut state = HistoryState::new();
        state.add_history("echo hi".to_string());
        assert_eq!(state.entries, vec!["echo hi".to_string()]);
    }

    #[test]
    fn load_history_reads_nonblank_trimmed_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "echo one\n  echo two  \n\n\necho three\n").unwrap();

        let mut state = HistoryState::new();
        state.load_history(path.to_str().unwrap()).unwrap();

        assert_eq!(
            state.entries,
            vec![
                "echo one".to_string(),
                "echo two".to_string(),
                "echo three".to_string(),
            ]
        );
    }

    #[test]
    fn load_history_missing_file_is_a_silent_no_op() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.txt");
        let mut state = HistoryState::new();
        assert!(state.load_history(path.to_str().unwrap()).is_ok());
        assert!(state.entries.is_empty());
    }

    #[test]
    fn load_history_reports_errors_other_than_not_found() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "echo one\n").unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o000);
        std::fs::set_permissions(&path, perms).unwrap();

        let mut state = HistoryState::new();
        let result = state.load_history(path.to_str().unwrap());

        // Restore permissions so TempDir can clean up the file.
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o644);
        std::fs::set_permissions(&path, perms).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn write_history_overwrites_the_whole_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "stale content that should be gone\n").unwrap();

        let mut state = HistoryState::new();
        state.add_history("echo one".to_string());
        state.add_history("echo two".to_string());
        state.write_history(path.to_str().unwrap()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "echo one\necho two\n");
    }

    #[test]
    fn append_history_only_writes_new_entries_since_last_append() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");

        let mut state = HistoryState::new();
        state.add_history("echo one".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();
        assert_eq!(state.last_appended_index, 1);

        state.add_history("echo two".to_string());
        state.add_history("echo three".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();
        assert_eq!(state.last_appended_index, 3);

        // Appending again with no new entries must not duplicate anything.
        state.append_history(path.to_str().unwrap()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "echo one\necho two\necho three\n");
    }

    #[test]
    fn append_history_preserves_pre_existing_file_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "echo previous-session\n").unwrap();

        let mut state = HistoryState::new();
        state.add_history("echo new".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "echo previous-session\necho new\n");
    }
}
