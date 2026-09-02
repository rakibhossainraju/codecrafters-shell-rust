use codecrafters_shell::commands;
use codecrafters_shell::editor::TerminalEditor;
use codecrafters_shell::error::ShellError;
use codecrafters_shell::parser::{Lexer, Parser};
use codecrafters_shell::state::ShellState;
use rustyline::error::ReadlineError;
use std::env;

fn main() {
    let mut editor = TerminalEditor::new();
    let mut state = ShellState::new();

    let history_file = env::var("HISTFILE").ok();

    if let Some(ref path) = history_file {
        match state.history.load_history(path) {
            Ok(_) => {
                for entry in &state.history.entries {
                    editor.add_history_entry(entry);
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    loop {
        let user_input = match editor.read_line() {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }
                state.history.add_history(input.clone());
                editor.add_history_entry(&input);
                input
            }
            Err(ShellError::Readline(ReadlineError::Eof)) => break,
            Err(ShellError::Readline(ReadlineError::Interrupted)) => break,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let tokens = match Lexer::tokenizer(&user_input) {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let ast = match Parser::parser(tokens) {
            Ok(ast_note) => ast_note,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        match commands::execute_ast(ast, &mut state) {
            Ok(_) => (),
            Err(ShellError::ExitOut) => break,
            Err(e) => eprintln!("{}", e),
        }
    }

    if let Some(ref path) = history_file {
        if let Err(e) = state.history.write_history(path) {
            eprintln!("{}", e);
        }
    }
}
