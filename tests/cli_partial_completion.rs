//! Feature under test: "Command Completion - Partial completions".
//!
//! When multiple matches share a common prefix *longer* than what's already
//! typed, <TAB> must first silently extend the input up to that longest
//! common prefix (LCP) -- no bell, no listing -- before ever falling back to
//! the "multiple matches" bell/list behavior from `cli_tab_completion.rs`.
//! The bell/list behavior only kicks in once the input already equals the
//! LCP and can't be extended any further.
//!
//! Official CodeCrafters example this stage is built around:
//!   PATH has `xyz_bee`, `xyz_bee_ant`, `xyz_bee_ant_dog`.
//!   Typed "xyz_", <TAB> -> line becomes "xyz_bee" (their shared prefix).

mod support;

use support::{BELL, Sandbox, expect_match_list, press_tab, type_str, wait_for_prompt};

#[test]
fn tester_example_partial_completes_then_bell_then_lists_all_three() {
    let sandbox = Sandbox::new();
    sandbox.add_executable("xyz_bee");
    sandbox.add_executable("xyz_bee_ant");
    sandbox.add_executable("xyz_bee_ant_dog");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "xyz_");

    // 1st <TAB>: all three share "xyz_bee" beyond what's typed -> extend to
    // it silently. No bell yet, since progress was made.
    press_tab(&mut session);
    let after_first_tab = session.exp_string("xyz_bee").unwrap();
    assert!(
        !after_first_tab.contains(BELL),
        "must not ring the bell while it can still extend the common prefix"
    );

    // Input now equals the full common prefix and 3 candidates remain
    // ("xyz_bee" itself, plus the two longer names) -- can't extend further,
    // so this is exactly the "multiple matches" case: bell on the next Tab.
    press_tab(&mut session);
    session.exp_char(BELL).unwrap();

    // And the one after that lists everything, sorted, space-separated.
    press_tab(&mut session);
    expect_match_list(&mut session, &["xyz_bee", "xyz_bee_ant", "xyz_bee_ant_dog"]);
    session.exp_string("$ xyz_bee").unwrap();
}

#[test]
fn partial_completion_extends_by_a_single_character() {
    let sandbox = Sandbox::new();
    // "e" avoids colliding with Sandbox's built-in fixture bins (argecho,
    // catit, upper, stderrer, failer, clear).
    sandbox.add_executable("elm");
    sandbox.add_executable("elms");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "el");

    press_tab(&mut session);
    let after_tab = session.exp_string("elm").unwrap();
    assert!(!after_tab.contains(BELL));
}

#[test]
fn partial_completion_stops_exactly_where_matches_diverge() {
    let sandbox = Sandbox::new();
    // Note: must not start with any letter used by Sandbox's own built-in
    // fixture bins (argecho, catit, upper, stderrer, failer, clear), or
    // they'd sneak into the candidate set and change the real LCP.
    sandbox.add_executable("kobdef");
    sandbox.add_executable("kobxyz");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "k");

    // Common prefix of "kobdef" and "kobxyz" is "kob" -- completion must
    // stop there, not overshoot into either full name.
    press_tab(&mut session);
    let after_tab = session.exp_string("kob").unwrap();
    assert!(!after_tab.contains(BELL));

    // Confirm it really did stop at "kob" and not silently complete further:
    // a second Tab (no further common prefix beyond "kob") should now ring
    // the bell rather than continuing to extend.
    press_tab(&mut session);
    session.exp_char(BELL).unwrap();
}

#[test]
fn partial_completion_also_applies_to_filename_and_path_completion() {
    let sandbox = Sandbox::new();
    sandbox.write_file("report_alpha.txt", "");
    sandbox.write_file("report_beta.txt", "");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "stat rep");

    // Shared prefix of the two filenames is "report_" -- same LCP logic
    // must apply on the path-completion branch, not just command names.
    press_tab(&mut session);
    let after_tab = session.exp_string("report_").unwrap();
    assert!(!after_tab.contains(BELL));
}

#[test]
fn no_common_prefix_beyond_typed_input_goes_straight_to_bell() {
    // Regression guard: when the candidates diverge immediately (nothing to
    // extend), behavior must be unchanged from the plain multiple-matches
    // case already covered in cli_tab_completion.rs -- no spurious partial
    // completion should be attempted.
    let sandbox = Sandbox::new();
    sandbox.add_executable("xyz_bee");
    sandbox.add_executable("xyz_cow");

    let mut session = sandbox.spawn_pty();
    wait_for_prompt(&mut session);
    type_str(&mut session, "xyz_");

    press_tab(&mut session);
    let before_bell = session.exp_char(BELL).unwrap();
    assert!(!before_bell.contains("xyz_bee") && !before_bell.contains("xyz_cow"));
}
