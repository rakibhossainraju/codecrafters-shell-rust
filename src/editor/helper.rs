use crate::commands::BUILTIN_COMMANDS;
use crate::utils::get_executable_names;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::fs;

pub struct EditorHelper;

impl Helper for EditorHelper {}
impl Highlighter for EditorHelper {}
impl Validator for EditorHelper {}
impl Hinter for EditorHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to_cursor = &line[..pos];
        
        // Find where the current word starts
        let start_idx = line_up_to_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let current_word = &line_up_to_cursor[start_idx..];

        let mut candidates = if start_idx == 0 {
            // Completing the first word: could be a builtin or external command
            let mut cmds = self.find_builtin_commands(current_word);
            cmds.extend(self.find_external_commands(current_word));
            
            // If there's only one command match, add a space
            if cmds.len() == 1 {
                cmds[0].replacement.push(' ');
            }
            cmds
        } else {
            // Completing subsequent words: assume it's a path
            self.find_path_completions(current_word)
        };

        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.display == b.display);

        Ok((start_idx, candidates))
    }
}

impl EditorHelper {
    fn find_builtin_commands(&self, input: &str) -> Vec<Pair> {
        BUILTIN_COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(input))
            .map(|(cmd, _)| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect()
    }

    fn find_external_commands(&self, input: &str) -> Vec<Pair> {
        get_executable_names()
            .into_iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: cmd,
            })
            .collect()
    }

    fn find_path_completions(&self, input: &str) -> Vec<Pair> {
        let (dir_to_scan, file_prefix) = match input.rfind('/') {
            Some(idx) => (&input[..idx + 1], &input[idx + 1..]),
            None => ("", input),
        };

        let scan_path = if dir_to_scan.is_empty() { "." } else { dir_to_scan };
        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(scan_path) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(file_prefix) {
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        let replacement = format!("{}{}{}", 
                            dir_to_scan, 
                            file_name, 
                            if is_dir { "/" } else { " " }
                        );
                        
                        candidates.push(Pair {
                            display: file_name,
                            replacement,
                        });
                    }
                }
            }
        }
        candidates
    }
}
