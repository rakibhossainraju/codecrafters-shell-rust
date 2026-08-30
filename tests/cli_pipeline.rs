mod support;

use support::{stdout, Sandbox};

#[test]
fn builtin_to_builtin_pipeline_only_the_last_stage_writes_to_real_stdout() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo a | echo b | echo c\n");
    assert_eq!(stdout(&out), "c\n");
}

#[test]
fn builtin_output_is_forwarded_into_an_external_command() {
    let sandbox = Sandbox::new();
    // echo's stdout is captured by the pipeline and fed into `upper`'s stdin.
    let out = sandbox.run("echo hello | upper\n");
    assert_eq!(stdout(&out), "HELLO\n");
}

#[test]
fn external_output_is_forwarded_into_a_builtin() {
    let sandbox = Sandbox::new();
    // `echo` ignores its stdin entirely, so the piped data has no effect on
    // its output, but the pipeline must still run to completion.
    let out = sandbox.run("argecho hi | echo bye\n");
    assert_eq!(stdout(&out), "bye\n");
}

#[test]
fn external_to_external_pipeline_forwards_real_data() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho hello world | upper\n");
    assert_eq!(stdout(&out), "HELLO\nWORLD\n");
}

#[test]
fn three_stage_mixed_pipeline() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho hello | upper | catit\n");
    assert_eq!(stdout(&out), "HELLO\n");
}

#[test]
fn pipeline_with_a_nonzero_exit_stage_does_not_hang_or_crash() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho x | failer | catit\necho still-alive\n");
    assert!(out.status.success());
    assert!(stdout(&out).contains("still-alive"));
}

#[test]
fn unknown_command_mid_pipeline_reports_not_found_and_shell_survives() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("argecho x | nosuchcmd | catit\necho still-alive\n");
    assert!(support::stderr(&out).contains("nosuchcmd: not found"));
    assert!(stdout(&out).contains("still-alive"));
    assert!(out.status.success());
}
