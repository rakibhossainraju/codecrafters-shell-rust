mod support;

use support::{Sandbox, stderr, stdout};

#[test]
fn single_quotes_preserve_literal_content() {
    let sandbox = Sandbox::new();
    let out = sandbox.run(concat!(r"argecho 'a\ b  c'", "\n"));
    // One literal argument, backslash and double space preserved verbatim.
    assert_eq!(stdout(&out), "a\\ b  c\n");
}

#[test]
fn double_quotes_allow_escaping_quote_and_backslash_only() {
    let sandbox = Sandbox::new();
    let out = sandbox.run(concat!(r#"argecho "say \"hi\" and \\ done""#, "\n"));
    assert_eq!(stdout(&out), "say \"hi\" and \\ done\n");
}

#[test]
fn unquoted_backslash_escapes_a_space_into_one_word() {
    let sandbox = Sandbox::new();
    let out = sandbox.run(concat!(r"argecho hello\ world", "\n"));
    assert_eq!(stdout(&out), "hello world\n");
}

#[test]
fn quoting_affects_word_splitting() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho unquoted \"one arg\" 'another arg'\n");
    assert_eq!(stdout(&out), "unquoted\none arg\nanother arg\n");
}

#[test]
fn unclosed_single_quote_is_reported_as_a_syntax_error() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo 'unterminated\necho still-alive\n");
    assert!(stderr(&out).contains("unclosed single quote"));
    assert!(stdout(&out).contains("still-alive"));
}

#[test]
fn unclosed_double_quote_is_reported_as_a_syntax_error() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo \"unterminated\necho still-alive\n");
    assert!(stderr(&out).contains("unclosed double quote"));
    assert!(stdout(&out).contains("still-alive"));
}

#[test]
fn trailing_pipe_is_a_syntax_error() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo hi |\necho still-alive\n");
    assert!(stderr(&out).contains("syntax error"));
    assert!(stdout(&out).contains("still-alive"));
}

/// Documents current (incomplete) behavior: `&&` is tokenized but the parser
/// has no And/Or handling, so only the first command runs and the rest of
/// the line is silently dropped -- no error, no second command execution.
#[test]
fn and_operator_is_not_implemented_and_silently_drops_the_second_command() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first && echo second\n");
    assert_eq!(stdout(&out), "first\n");
    assert_eq!(stderr(&out), "");
}

/// Same gap for `||`.
#[test]
fn or_operator_is_not_implemented_and_silently_drops_the_second_command() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first || echo second\n");
    assert_eq!(stdout(&out), "first\n");
}

/// Unlike `&&`/`||` above, `&` is implemented: it backgrounds the command
/// before it and continues on to the rest of the line. Backgrounding a
/// builtin spawns a real re-exec'd child process to run it (see
/// `executors::background::spawn_builtin_job`), so its output genuinely
/// races with the foreground command that follows -- unlike when builtins
/// ran in-process and synchronously, "first" vs "second" is no longer
/// deterministically ordered. What *is* guaranteed: the job announcement
/// (`[1] <pid>`) prints synchronously, before the shell moves on to
/// `echo second`, and both commands' output eventually shows up.
///
/// Not asserted: an exact line count after removing the announcement.
/// `main`'s REPL loop calls `print_done_job()` once after this whole line
/// finishes, and if the backgrounded child happens to have already exited
/// by then, a trailing "done" notification can show up too -- benign extra
/// output, not something this test cares about either way.
#[test]
fn background_operator_runs_the_preceding_command_then_continues() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first & echo second\n");
    let captured = stdout(&out);
    let lines: Vec<&str> = captured.lines().collect();

    let announcement_pos = lines
        .iter()
        .position(|line| line.starts_with("[1] "))
        .expect("background job announcement should be printed");
    let second_pos = lines
        .iter()
        .position(|&line| line == "second")
        .expect("foreground command's output should be printed");
    assert!(
        announcement_pos < second_pos,
        "job announcement must print before the shell moves on to the foreground command"
    );

    assert!(lines.contains(&"first"));
    assert!(lines.contains(&"second"));

    assert_eq!(stderr(&out), "");
}
