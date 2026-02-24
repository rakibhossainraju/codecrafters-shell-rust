use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use crate::commands::BUILTIN_COMMANDS;

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

        for builtin in BUILTIN_COMMANDS {
            if builtin.starts_with(input) {
                candidates.push(Pair {
                    display: builtin.to_string(),
                    replacement: format!("{} ", builtin),
                });
            }
        }
        Ok((0, candidates))
    }
}
