use crate::error::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufRead, BufReader};

pub struct ShellState {
    pub history: Vec<String>,
    pub last_history_appended_index: usize,
    // In the future:
    // pub env_vars: HashMap<String, String>,
    // pub aliases: HashMap<String, String>,
    // pub last_exit_code: Option<i32>,
}
impl Default for ShellState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            last_history_appended_index: 0,
        }
    }
}

impl ShellState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_history(&mut self, cmd: String) {
        self.history.push(cmd);
    }

    pub fn load_history(&mut self, filename: &str) -> Result<()> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                self.history.push(trimmed.to_string());
            }
        }
        Ok(())
    }

    pub fn write_history(&self, filename: &str) -> Result<()> {
        use std::io::Write;
        let mut file = File::create(filename)?;
        for cmd in &self.history {
            writeln!(file, "{}", cmd)?;
        }
        Ok(())
    }

    pub fn append_history(&mut self, filename: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;

        let start_index = self.last_history_appended_index;
        let end_index = self.history.len();

        for i in start_index..end_index {
            writeln!(file, "{}", self.history[i])?;
        }

        self.last_history_appended_index = end_index;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn new_state_has_empty_history() {
        let state = ShellState::new();
        assert!(state.history.is_empty());
        assert_eq!(state.last_history_appended_index, 0);
    }

    #[test]
    fn add_history_pushes_a_single_entry() {
        let mut state = ShellState::new();
        state.add_history("echo hi".to_string());
        assert_eq!(state.history, vec!["echo hi".to_string()]);
    }

    #[test]
    fn load_history_reads_nonblank_trimmed_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "echo one\n  echo two  \n\n\necho three\n").unwrap();

        let mut state = ShellState::new();
        state.load_history(path.to_str().unwrap()).unwrap();

        assert_eq!(
            state.history,
            vec![
                "echo one".to_string(),
                "echo two".to_string(),
                "echo three".to_string(),
            ]
        );
    }

    #[test]
    fn load_history_missing_file_is_an_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.txt");
        let mut state = ShellState::new();
        assert!(state.load_history(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn write_history_overwrites_the_whole_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hist.txt");
        std::fs::write(&path, "stale content that should be gone\n").unwrap();

        let mut state = ShellState::new();
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

        let mut state = ShellState::new();
        state.add_history("echo one".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();
        assert_eq!(state.last_history_appended_index, 1);

        state.add_history("echo two".to_string());
        state.add_history("echo three".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();
        assert_eq!(state.last_history_appended_index, 3);

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

        let mut state = ShellState::new();
        state.add_history("echo new".to_string());
        state.append_history(path.to_str().unwrap()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "echo previous-session\necho new\n");
    }
}
