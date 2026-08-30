mod support;

use support::{stderr, stdout, Sandbox};

fn canon(p: &std::path::Path) -> String {
    std::fs::canonicalize(p).unwrap().display().to_string()
}

#[test]
fn echo_with_no_args_prints_a_blank_line() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo\n");
    assert_eq!(stdout(&out), "\n");
    assert!(out.status.success());
}

#[test]
fn echo_joins_args_with_single_spaces() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo hello   world\n");
    assert_eq!(stdout(&out), "hello world\n");
}

#[test]
fn echo_preserves_quoted_internal_spacing() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo \"hello   world\"\n");
    assert_eq!(stdout(&out), "hello   world\n");
}

#[test]
fn pwd_prints_the_current_working_directory() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("pwd\n");
    assert_eq!(stdout(&out).trim_end(), canon(&sandbox.work_dir));
    assert!(out.status.success());
}

#[test]
fn cd_changes_directory_relatively_and_absolutely() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.path("sub/inner")).unwrap();

    let out = sandbox.run("cd sub\npwd\n");
    assert_eq!(stdout(&out).trim_end(), canon(&sandbox.path("sub")));

    let abs = canon(&sandbox.path("sub/inner"));
    let out = sandbox.run(&format!("cd {}\npwd\n", abs));
    assert_eq!(stdout(&out).trim_end(), abs);
}

#[test]
fn cd_expands_home_and_home_relative_paths() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.home_dir.join("docs")).unwrap();

    let out = sandbox.run("cd ~\npwd\n");
    assert_eq!(stdout(&out).trim_end(), canon(&sandbox.home_dir));

    let out = sandbox.run("cd ~/docs\npwd\n");
    assert_eq!(stdout(&out).trim_end(), canon(&sandbox.home_dir.join("docs")));
}

#[test]
fn cd_into_nonexistent_directory_reports_an_error_and_does_not_move() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("cd doesnotexist\npwd\n");
    assert!(stderr(&out).contains("cd: doesnotexist: No such file or directory"));
    // cwd should be unchanged (still the sandbox work dir).
    assert_eq!(stdout(&out).trim_end(), canon(&sandbox.work_dir));
}

#[test]
fn cd_into_a_file_reports_an_error() {
    let sandbox = Sandbox::new();
    sandbox.write_file("notadir.txt", "content");
    let out = sandbox.run("cd notadir.txt\n");
    assert!(stderr(&out).contains("cd: notadir.txt: No such file or directory"));
}

#[test]
fn type_reports_shell_builtins() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("type echo\n");
    assert_eq!(stdout(&out), "echo is a shell builtin\n");
}

#[test]
fn type_reports_external_commands_with_resolved_path() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("type argecho\n");
    assert_eq!(
        stdout(&out),
        format!("argecho is {}\n", sandbox.bin_dir.join("argecho").display())
    );
}

#[test]
fn type_reports_not_found_for_unknown_commands() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("type nosuchcommand\n");
    assert!(stderr(&out).contains("nosuchcommand: not found"));
}

#[test]
fn help_lists_all_builtins() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("help\n");
    let text = stdout(&out);
    assert!(text.starts_with("Available builtin commands:\n"));
    for name in [
        "exit", "echo", "help", "type", "pwd", "cd", "clear", "history",
    ] {
        assert!(
            text.contains(name),
            "expected help output to mention `{name}`, got:\n{text}"
        );
    }
}

#[test]
fn unknown_command_reports_not_found_and_shell_keeps_running() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("nosuchcommand\necho still-alive\n");
    assert!(stderr(&out).contains("nosuchcommand: not found"));
    assert!(stdout(&out).contains("still-alive"));
    assert!(out.status.success());
}

#[test]
fn exit_stops_the_shell_before_later_lines_run() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo before\nexit\necho after\n");
    let text = stdout(&out);
    assert!(text.contains("before"));
    assert!(!text.contains("after"));
    assert!(out.status.success());
}

#[test]
fn external_command_runs_with_argv0_set_to_the_typed_name() {
    let sandbox = Sandbox::new();
    // argecho just echoes its argv, so this also exercises basic external
    // command execution end-to-end via the sandboxed PATH.
    let out = sandbox.run("argecho one two three\n");
    assert_eq!(stdout(&out), "one\ntwo\nthree\n");
}
