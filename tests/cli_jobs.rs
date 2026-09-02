mod support;

use support::{Sandbox, stderr, stdout};

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
