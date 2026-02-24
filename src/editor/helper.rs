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
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let candidates: Vec<Pair> = Vec::new();
        // TODO: autocomplete
        // `line` is what the user has typed so far (e.g., "ech")
        // `pos` is where their cursor currently is.

        // 1. Check if the line matches the beginning of "echo" or "exit".
        // 2. If it does, create a Pair and push it to candidates:
        //    Pair {
        //        display: "echo".to_string(),
        //        replacement: "echo ".to_string() // <-- Notice the trailing space!
        //    }

        // The '0' means we are replacing the string starting from index 0 of the input.

        Ok((0, candidates))
    }
}
