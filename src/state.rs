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
}
