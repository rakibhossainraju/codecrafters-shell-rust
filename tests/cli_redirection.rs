mod support;

use support::{stderr, stdout, Sandbox};

#[test]
fn output_redirect_creates_and_truncates_the_target_file() {
    let sandbox = Sandbox::new();
    sandbox.write_file("out.txt", "stale data that should be gone\n");
    sandbox.run("echo fresh > out.txt\n");
    assert_eq!(sandbox.read_file("out.txt"), "fresh\n");
}

#[test]
fn output_redirect_suppresses_normal_stdout() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo fresh > out.txt\n");
    assert_eq!(stdout(&out), "");
}

#[test]
fn append_redirect_keeps_prior_content() {
    let sandbox = Sandbox::new();
    sandbox.run("echo first > out.txt\necho second >> out.txt\n");
    assert_eq!(sandbox.read_file("out.txt"), "first\nsecond\n");
}

#[test]
fn explicit_stdout_descriptor_behaves_like_plain_redirect() {
    let sandbox = Sandbox::new();
    sandbox.run("echo hi 1> out.txt\n");
    assert_eq!(sandbox.read_file("out.txt"), "hi\n");
}

#[test]
fn stderr_descriptor_redirects_only_stderr() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("stderrer 2> err.txt\n");
    assert_eq!(sandbox.read_file("err.txt"), "err message\n");
    assert_eq!(stdout(&out), "");
}

#[test]
fn explicit_stdin_descriptor_feeds_a_file_into_an_external_command() {
    let sandbox = Sandbox::new();
    sandbox.write_file("in.txt", "line one\nline two\n");
    let out = sandbox.run("catit 0< in.txt\n");
    assert_eq!(stdout(&out), "line one\nline two\n");
}

#[test]
fn stdin_and_stdout_redirection_combine_on_the_same_command() {
    let sandbox = Sandbox::new();
    sandbox.write_file("in.txt", "round trip content\n");
    sandbox.run("catit 0< in.txt > out.txt\n");
    assert_eq!(sandbox.read_file("out.txt"), "round trip content\n");
}

#[test]
fn redirect_to_an_unwritable_path_reports_an_error_and_shell_keeps_running() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo hi > nosuchdir/out.txt\necho still-alive\n");
    assert!(stderr(&out).contains("nosuchdir/out.txt"));
    assert!(stdout(&out).contains("still-alive"));
    assert!(out.status.success());
}
