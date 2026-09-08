mod support;

use support::{Sandbox, stderr, stdout};
use std::time::{Duration, Instant};

fn canon(p: &std::path::Path) -> String {
    std::fs::canonicalize(p).unwrap().display().to_string()
}

#[test]
fn type_reports_jobs_as_a_shell_builtin() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("type jobs\n");
    assert_eq!(stdout(&out), "jobs is a shell builtin\n");
}

#[test]
fn jobs_with_no_background_jobs_produces_no_output() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("jobs\n");
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), "");
    assert!(out.status.success());
}

#[test]
fn jobs_returns_to_the_prompt_and_shell_keeps_running() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("jobs\necho still-alive\n");
    assert_eq!(stdout(&out), "still-alive\n");
    assert!(out.status.success());
}

/// Backgrounding a builtin (`echo hi &`) has no separate OS process to hand
/// off the way an external command does, so it re-execs the shell binary
/// itself as a child (see `executors::background::spawn_builtin_job`). That
/// child gets pushed into the job table exactly like an external command,
/// so the usual `[<id>] <pid>` announcement should print for it too.
///
/// The announcement (printed synchronously by the parent) and the child's
/// own "hi" both land on the same unredirected stdout pipe here, so their
/// relative order isn't guaranteed under scheduling pressure -- search for
/// the announcement instead of assuming it's the first line.
#[test]
fn backgrounding_a_builtin_prints_a_job_announcement_with_a_real_pid() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo hi &\n");

    let captured = stdout(&out);
    let announcement = captured
        .lines()
        .find(|line| line.starts_with("[1] "))
        .expect("should print a job announcement")
        .to_string();
    let pid_str = announcement
        .strip_prefix("[1] ")
        .expect("announcement should be formatted as `[<job_id>] <pid>`");
    let pid: u32 = pid_str.parse().expect("pid should be a plain number");
    assert!(pid > 0);
}

/// A backgrounded builtin's own redirections must still be honored -- the
/// parent resolves them once (`spawn_builtin_job`) and wires them into the
/// re-exec'd child's real stdio, so the child never sees `> out.txt` at all,
/// it just inherits an already-redirected file descriptor.
///
/// The write itself happens asynchronously in that child process, so this
/// polls briefly rather than assuming it's already landed the moment the
/// parent shell (and this call) returns.
#[test]
fn backgrounding_a_builtin_still_honors_its_own_redirection() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo redirected > out.txt &\n");

    // Nothing but the job announcement should reach the shell's own stdout;
    // the builtin's actual output went to the file instead.
    let captured = stdout(&out);
    assert!(captured.starts_with("[1] "));
    assert!(!captured.contains("redirected"));

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if sandbox.file_exists("out.txt") && sandbox.read_file("out.txt") == "redirected\n" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "backgrounded builtin's redirected output never landed in the file"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Backgrounding a builtin re-execs a whole separate process, so it runs
/// with its own copy of shell state, not the parent's -- exactly like a real
/// shell's subshell semantics for `&`. `cd sub &` changes the *child's*
/// working directory only; the parent's `pwd` must still show the original
/// directory, unconditionally and regardless of the child's timing (its
/// effect can never reach back into this process).
///
/// Not asserted on position: `main`'s REPL loop calls `print_done_job()`
/// after every line, including right after backgrounding `cd sub &` and
/// again after `pwd` -- if the backgrounded child happens to finish inside
/// that window, a "done" notification can land between the announcement and
/// `pwd`'s output, or even after it. Presence, not position, is what's
/// actually guaranteed here.
#[test]
fn backgrounding_cd_does_not_affect_the_parent_shells_directory() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.path("sub")).unwrap();

    let out = sandbox.run("cd sub &\npwd\n");
    let captured = stdout(&out);
    let lines: Vec<&str> = captured.lines().collect();

    assert!(
        lines.iter().any(|l| l.starts_with("[1] ")),
        "expected a job announcement somewhere, got: {:?}",
        lines
    );
    let expected_pwd = canon(&sandbox.work_dir);
    assert!(
        lines.iter().any(|&l| l == expected_pwd),
        "expected pwd's output ({}) somewhere in: {:?}",
        expected_pwd,
        lines
    );
}
