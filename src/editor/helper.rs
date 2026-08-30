use crate::commands::BUILTIN_COMMANDS;
use crate::utils::get_executable_names;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::cell::RefCell;
use std::io::Write;
use std::{fs, io};

pub struct EditorHelper {
    pub last_tab_state: RefCell<Option<(String, usize)>>,
}

impl Helper for EditorHelper {}
impl Highlighter for EditorHelper {}
impl Validator for EditorHelper {}
impl Hinter for EditorHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl EditorHelper {
    pub fn new() -> Self {
        EditorHelper {
            last_tab_state: RefCell::new(None),
        }
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

        // 1. Check if the user hit TAB twice in a row without typing anything else.
        let current_state = (line_up_to_cursor.to_string(), pos);
        let is_double_tab = {
            let mut state = self.last_tab_state.borrow_mut();
            let matched = state.as_ref() == Some(&current_state);
            *state = Some(current_state);
            matched
        };

        // Find where the current word starts
        let start_idx = line_up_to_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let current_word = &line_up_to_cursor[start_idx..];

        let candidates = if start_idx == 0 {
            // Completing the first word: could be a builtin or external command
            let mut cmds = self.find_builtin_commands(current_word);
            cmds.extend(self.find_external_commands(current_word));

            // Deduplicate before checking length
            cmds.sort_by(|a, b| a.display.cmp(&b.display));
            cmds.dedup_by(|a, b| a.display == b.display);

            // If there's only one command match, add a space
            if cmds.len() == 1 {
                cmds[0].replacement.push(' ');
            }
            cmds
        } else {
            // Completing subsequent words: assume it's a path
            let mut paths = self.find_path_completions(current_word);
            paths.sort_by(|a, b| a.display.cmp(&b.display));
            paths.dedup_by(|a, b| a.display == b.display);
            paths
        };

        // 2. If 0 or 1 match. let rustline handle it normally.
        if candidates.len() <= 1 {
            return Ok((start_idx, candidates));
        }

        // 3. -- MULTIPLE MATCHES INTERCEPT ---
        if is_double_tab {
            // Second <TAB>: Prinit the list manually
            let output = candidates
                .iter()
                .map(|c| c.display.clone())
                .collect::<Vec<_>>()
                .join("  "); // Two spaces between matches as recommended
            // Print a newline, the list, another newline, and manually redraw the prompt
            print!("\n{}\n$ {}", output, line);
            io::stdout().flush()?;
        } else {
            // First <TAB>: Ring the bell
            print!("\x07");
            io::stdout().flush().unwrap();
        }

        // Return empty candidates!
        // This stops rustyline from doing its own formatting and ruining our beautiful output.
        Ok((start_idx, vec![]))
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

        let scan_path = if dir_to_scan.is_empty() {
            "."
        } else {
            dir_to_scan
        };
        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(scan_path) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(file_prefix) {
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                        // ADD THE SLASH TO THE DISPLAY NAME IF IT'S A DIR
                        let display_name =
                            format!("{}{}", file_name, if is_dir { "/" } else { "" });

                        let replacement = format!(
                            "{}{}{}",
                            dir_to_scan,
                            file_name,
                            if is_dir { "/" } else { " " }
                        );

                        candidates.push(Pair {
                            display: display_name,
                            replacement,
                        });
                    }
                }
            }
        }
        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn names(pairs: &[Pair]) -> Vec<String> {
        let mut v: Vec<String> = pairs.iter().map(|p| p.display.clone()).collect();
        v.sort();
        v
    }

    #[test]
    fn find_builtin_commands_matches_by_prefix() {
        let helper = EditorHelper::new();
        assert_eq!(names(&helper.find_builtin_commands("ec")), vec!["echo"]);
        assert_eq!(
            names(&helper.find_builtin_commands("")),
            {
                let mut all: Vec<String> =
                    crate::commands::BUILTIN_COMMANDS.iter().map(|(n, _)| n.to_string()).collect();
                all.sort();
                all
            }
        );
        assert!(helper.find_builtin_commands("zzz-nope").is_empty());
    }

    #[test]
    fn find_external_commands_matches_only_whats_on_path() {
        let dir = TempDir::new().unwrap();
        let exe_path = dir.path().join("myfakecmd");
        std::fs::write(&exe_path, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = std::fs::metadata(&exe_path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        std::fs::set_permissions(&exe_path, perms).unwrap();

        let original_path = env::var_os("PATH");
        unsafe { env::set_var("PATH", dir.path()) };

        let helper = EditorHelper::new();
        let found = names(&helper.find_external_commands("myfake"));

        match original_path {
            Some(v) => unsafe { env::set_var("PATH", v) },
            None => unsafe { env::remove_var("PATH") },
        }

        assert_eq!(found, vec!["myfakecmd".to_string()]);
    }

    #[test]
    fn find_path_completions_lists_files_and_marks_directories() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("file_a.txt"), "").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let helper = EditorHelper::new();
        let candidates = helper.find_path_completions("");

        env::set_current_dir(original_cwd).unwrap();

        let file_entry = candidates
            .iter()
            .find(|p| p.display == "file_a.txt")
            .expect("file_a.txt should be found");
        assert_eq!(file_entry.replacement, "file_a.txt ");

        let dir_entry = candidates
            .iter()
            .find(|p| p.display == "subdir/")
            .expect("subdir should be found and marked as a directory");
        assert_eq!(dir_entry.replacement, "subdir/");
    }

    #[test]
    fn find_path_completions_filters_by_prefix_within_a_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested").join("apple.txt"), "").unwrap();
        std::fs::write(dir.path().join("nested").join("banana.txt"), "").unwrap();

        let helper = EditorHelper::new();
        let prefix = format!("{}/a", dir.path().join("nested").display());
        let candidates = helper.find_path_completions(&prefix);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].display, "apple.txt");
    }
}
