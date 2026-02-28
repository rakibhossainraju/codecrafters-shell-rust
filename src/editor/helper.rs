use crate::commands::BUILTIN_COMMANDS;
use crate::utils::get_executable_names;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

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
        let mut candidates: Vec<Pair> = Vec::new();

        let input = line[..pos].trim_end();

        if input.is_empty() {
            return Ok((0, candidates));
        }
        candidates.extend(self.find_builtin_commands(input));
        if candidates.is_empty() {
            candidates.extend(self.find_external_commands(input));
        }
        if candidates.len() == 1 {
            candidates[0].replacement.push(' ');
        }

        Ok((0, candidates))
    }
}

impl EditorHelper {
    fn find_builtin_commands(&self, input: &str) -> Vec<Pair> {
        BUILTIN_COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(input))
            .map(|(cmd, _)| Pair {
                display: cmd.to_string(),
                replacement: format!("{}", cmd),
            })
            .collect()
    }

    fn find_external_commands(&self, input: &str) -> Vec<Pair> {
        get_executable_names()
            .iter()
            .filter(|&cmd| cmd.starts_with(input))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: format!("{}", cmd),
            })
            .collect()
    }
}
