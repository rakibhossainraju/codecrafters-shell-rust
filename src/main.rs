#![allow(dead_code, unused_variables)]

mod commands;
mod editor;
mod error;
mod parser;
mod utils;

use crate::editor::TerminalEditor;
use crate::error::ShellError;
use crate::parser::{Lexer, Parser};
use commands::{BuiltinCommands, Command};
use rustyline::error::ReadlineError;

fn main() {
    loop {
        // Read user input and trim trailing whitespace
        let mut editor = TerminalEditor::new();
        let user_input = match editor.read_line() {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }
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

        let parsed_cmd = match Parser::parser(tokens) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let cmd = match Command::resolve(parsed_cmd) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        // Check for exit before executing (to break the loop)
        if matches!(cmd, Command::Builtin(BuiltinCommands::Exit, _)) {
            break;
        }

        // Execute the command with remaining arguments
        if let Err(e) = cmd.execute() {
            eprintln!("{}", e);
        }
    }
}
