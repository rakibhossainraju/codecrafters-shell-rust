mod support;

use support::{Sandbox, stdout};

#[test]
fn history_with_no_args_shows_up_to_last_ten_entries_numbered() {
    let sandbox = Sandbox::new();
    let out = sandbox.run("echo one\necho two\necho three\nhistory\n");
    let text = stdout(&out);
    assert!(text.contains("    1  echo one\n"));
    assert!(text.contains("    2  echo two\n"));
    assert!(text.contains("    3  echo three\n"));
    // The `history` line itself was appended to history before running.
    assert!(text.contains("    4  history\n"));
}

#[test]
fn history_with_numeric_arg_limits_to_last_n_entries() {
    let sandbox = Sandbox::new();
    // The `history 2` line itself is appended to history before it runs, so
    // the last 2 entries at execution time are "echo three" and the
    // "history 2" invocation itself -- "echo two" has already scrolled out.
    let out = sandbox.run("echo one\necho two\necho three\nhistory 2\n");
    let text = stdout(&out);
    assert!(!text.contains("echo one"));
    assert!(!text.contains("echo two"));
    assert!(text.contains("echo three"));
    assert!(text.contains("history 2"));
}

#[test]
fn history_write_flag_writes_all_entries_to_a_file() {
    let sandbox = Sandbox::new();
    sandbox.run("echo one\necho two\nhistory -w saved.txt\n");
    let contents = sandbox.read_file("saved.txt");
    assert_eq!(contents, "echo one\necho two\nhistory -w saved.txt\n");
}

#[test]
fn history_read_flag_loads_entries_from_a_file() {
    let sandbox = Sandbox::new();
    sandbox.write_file("preset.txt", "echo from-file-one\necho from-file-two\n");
    let out = sandbox.run("history -r preset.txt\nhistory\n");
    let text = stdout(&out);
    assert!(text.contains("echo from-file-one"));
    assert!(text.contains("echo from-file-two"));
}

#[test]
fn history_append_flag_only_appends_new_entries_once() {
    let sandbox = Sandbox::new();
    sandbox.run("echo one\nhistory -a appended.txt\necho two\nhistory -a appended.txt\n");
    let contents = sandbox.read_file("appended.txt");
    // First append writes "echo one" + the `history -a` line itself; the
    // second append should only add what's new since then, not repeat it.
    assert_eq!(
        contents,
        "echo one\nhistory -a appended.txt\necho two\nhistory -a appended.txt\n"
    );
}

#[test]
fn histfile_env_var_is_loaded_on_start_and_saved_on_exit() {
    let sandbox = Sandbox::new();

    // First session: build up some history, then exit normally so it's
    // flushed to HISTFILE.
    let out1 = sandbox.run("echo session-one-a\necho session-one-b\nexit\n");
    assert!(out1.status.success());
    assert!(sandbox.histfile.exists());

    // Second, independent process should load that same HISTFILE on start.
    let out2 = sandbox.run("history\n");
    let text = stdout(&out2);
    assert!(text.contains("echo session-one-a"));
    assert!(text.contains("echo session-one-b"));
}
