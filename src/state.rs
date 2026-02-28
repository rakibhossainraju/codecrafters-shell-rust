use crate::error::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct ShellState {
    pub history: Vec<String>,
    // In the future:
    // pub env_vars: HashMap<String, String>,
    // pub aliases: HashMap<String, String>,
    // pub last_exit_code: Option<i32>,
}
impl Default for ShellState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
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
}
