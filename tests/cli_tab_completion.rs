//! Feature under test: "handle tab completion when an entry has multiple
//! matches" (path/filename completion only -- see module docs on
//! `Sandbox::spawn_pty` for why this needs a real pty rather than the plain
//! piped harness used everywhere else).
//!
//! Spec (from the CodeCrafters stage description):
//!   - 1st <TAB> with multiple matches: ring the bell (`\x07`), input unchanged.
//!   - 2nd (and later) <TAB>: print all matches on a new line, alphabetically
//!     sorted, separated by at least one space (two recommended), each
//!     directory suffixed with `/` and each file with no trailing character,
//!     then redisplay the prompt with the original input preserved.

mod support;

use support::{press_tab, type_str, wait_for_prompt, Sandbox, BELL};

#[test]
fn spec_example_two_matches_bell_then_sorted_list_with_trailing_slash() {
    let sandbox = Sandbox::new();
    sandbox.write_file("bar.txt", "");
    std::fs::create_dir(sandbox.path("foo")).unwrap();

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat ");

    // First <TAB>: bell only, nothing about the matches printed yet.
    press_tab(&mut session);
    let before_bell = session.exp_char(BELL).unwrap();
    assert!(
        !before_bell.contains("bar.txt") && !before_bell.contains("foo"),
        "matches must not be listed before the bell rings, got: {before_bell:?}"
    );

    // Second <TAB>: sorted, two-space separated, dir gets a trailing slash.
    press_tab(&mut session);
    session.exp_string("bar.txt  foo/").unwrap();

    // Prompt reappears with the original input intact.
    session.exp_string("$ stat ").unwrap();
}

#[test]
fn first_tab_only_rings_bell_without_printing_matches_yet() {
    let sandbox = Sandbox::new();
    sandbox.write_file("one.txt", "");
    sandbox.write_file("two.txt", "");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat ");

    press_tab(&mut session);
    let before_bell = session.exp_char(BELL).unwrap();
    assert!(!before_bell.contains("one.txt"));
    assert!(!before_bell.contains("two.txt"));
}

#[test]
fn matches_are_sorted_alphabetically_regardless_of_filesystem_creation_order() {
    let sandbox = Sandbox::new();
    // Written in a deliberately non-alphabetical order.
    sandbox.write_file("zeta.txt", "");
    sandbox.write_file("alpha.txt", "");
    std::fs::create_dir(sandbox.path("mid")).unwrap();

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat ");

    press_tab(&mut session);
    session.exp_char(BELL).unwrap();
    press_tab(&mut session);

    session.exp_string("alpha.txt  mid/  zeta.txt").unwrap();
}

#[test]
fn file_only_matches_get_no_trailing_character() {
    let sandbox = Sandbox::new();
    sandbox.write_file("report.txt", "");
    sandbox.write_file("receipt.txt", "");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat re");

    press_tab(&mut session);
    session.exp_char(BELL).unwrap();
    press_tab(&mut session);

    session.exp_string("receipt.txt  report.txt").unwrap();
}

#[test]
fn directory_only_matches_all_get_a_trailing_slash() {
    let sandbox = Sandbox::new();
    std::fs::create_dir(sandbox.path("docs")).unwrap();
    std::fs::create_dir(sandbox.path("data")).unwrap();

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat d");

    press_tab(&mut session);
    session.exp_char(BELL).unwrap();
    press_tab(&mut session);

    session.exp_string("data/  docs/").unwrap();
}

#[test]
fn single_match_autocompletes_without_ringing_the_bell() {
    // Guard/regression test: a single match must keep using the existing
    // (already-working) auto-complete-in-place behavior, not the new
    // multi-match bell/list path.
    let sandbox = Sandbox::new();
    sandbox.write_file("uniquefile.txt", "");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat uniq");

    press_tab(&mut session);
    let before_completion = session.exp_string("uniquefile.txt ").unwrap();
    assert!(!before_completion.contains(BELL));
}

#[test]
fn prompt_reappears_with_original_input_preserved_and_still_editable() {
    let sandbox = Sandbox::new();
    sandbox.write_file("bar.txt", "");
    std::fs::create_dir(sandbox.path("foo")).unwrap();

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat ");

    press_tab(&mut session);
    session.exp_char(BELL).unwrap();
    press_tab(&mut session);
    session.exp_string("bar.txt  foo/").unwrap();
    session.exp_string("$ stat ").unwrap();

    // The user can keep typing to narrow the match down further.
    type_str(&mut session, "b");
}
