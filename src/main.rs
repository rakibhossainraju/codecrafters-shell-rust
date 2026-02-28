#![allow(dead_code, unused_variables, unused_imports)]

mod commands;
mod editor;
mod error;
mod parser;
mod state;
mod utils;

use crate::editor::TerminalEditor;
use crate::error::ShellError;
use crate::parser::{Lexer, Parser};
use crate::state::ShellState;
use rustyline::error::ReadlineError;

fn main() {
    let mut editor = TerminalEditor::new();
    let mut state = ShellState::new();
    loop {
        let user_input = match editor.read_line() {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }
                state.history.push(input.clone());
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
}
