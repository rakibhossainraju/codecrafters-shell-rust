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

#[test]
fn and_runs_the_right_side_when_the_left_side_succeeds() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first && echo second\n");
    assert_eq!(stdout(&out), "first\nsecond\n");
    assert_eq!(stderr(&out), "");
}

/// `failer` (a fixture bin, see `Sandbox::install_fixture_bins`) exits 3
/// with no output -- the left side ran, but failed, so the right side must
/// not run at all.
#[test]
fn and_skips_the_right_side_when_the_left_side_exits_nonzero() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("failer && echo second\n");
    assert_eq!(stdout(&out), "");
}

/// A left side that doesn't even resolve (command not found) counts as a
/// failure for `&&` purposes too -- the error still prints, the right side
/// still gets skipped, and unlike a hard parser/IO error the shell survives
/// to run the next line normally.
#[test]
fn and_skips_the_right_side_when_the_left_side_is_not_found() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("nosuchcmd && echo second\necho still-alive\n");
    assert!(stderr(&out).contains("nosuchcmd"));
    assert_eq!(stdout(&out), "still-alive\n");
}

#[test]
fn and_chains_more_than_two_commands() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo a && echo b && echo c\n");
    assert_eq!(stdout(&out), "a\nb\nc\n");
}

#[test]
fn or_skips_the_right_side_when_the_left_side_succeeds() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first || echo second\n");
    assert_eq!(stdout(&out), "first\n");
    assert_eq!(stderr(&out), "");
}

#[test]
fn or_runs_the_right_side_when_the_left_side_exits_nonzero() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("failer || echo second\n");
    assert_eq!(stdout(&out), "second\n");
    assert_eq!(stderr(&out), "");
}

#[test]
fn or_runs_the_right_side_when_the_left_side_is_not_found() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("nosuchcmd || echo second\necho still-alive\n");
    assert!(stderr(&out).contains("nosuchcmd"));
    assert_eq!(stdout(&out), "second\nstill-alive\n");
}

#[test]
fn or_chains_more_than_two_commands() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("failer || failer || echo c\n");
    assert_eq!(stdout(&out), "c\n");
}

#[test]
fn mixed_and_or_is_left_associative_and_runs_in_order() {
    let sandbox = Sandbox::new();
    // (echo a || echo b) && echo c -> a succeeds, b skipped, c runs
    let out1 = sandbox.run("echo a || echo b && echo c\n");
    assert_eq!(stdout(&out1), "a\nc\n");

    // (failer && echo b) || echo c -> failer fails, b skipped, c runs
    let out2 = sandbox.run("failer && echo b || echo c\n");
    assert_eq!(stdout(&out2), "c\n");

    // (echo a && failer) || echo c -> a runs, failer fails, c runs
    let out3 = sandbox.run("echo a && failer || echo c\n");
    assert_eq!(stdout(&out3), "a\nc\n");

    // (failer || echo b) && echo c -> failer fails, b runs, c runs
    let out4 = sandbox.run("failer || echo b && echo c\n");
    assert_eq!(stdout(&out4), "b\nc\n");
}

#[test]
fn or_chains_whole_pipelines_not_just_simple_commands() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho hi | upper || echo fallback\n");
    assert_eq!(stdout(&out), "HI\n");
}

#[test]
fn trailing_or_is_a_syntax_error() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo first ||\n");
    assert!(stderr(&out).contains("unexpected"));
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
