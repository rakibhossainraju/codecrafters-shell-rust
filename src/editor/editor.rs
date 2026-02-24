use crate::editor::EditorHelper;
use crate::error::Result;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

pub struct TerminalEditor {
    rl: Editor<EditorHelper, DefaultHistory>,
}

impl TerminalEditor {
    pub fn new() -> Self {
        let mut rl = Editor::new().expect("Failed to initialize editor");
        rl.set_helper(Some(EditorHelper));
        TerminalEditor { rl }
    }

    pub fn read_line(&mut self) -> Result<String> {
        // The? operator automatically converts ReadlineError into ShellError::Readline
        let user_input = self.rl.readline("$ ")?;
        Ok(user_input.trim().to_string())
    }
}
