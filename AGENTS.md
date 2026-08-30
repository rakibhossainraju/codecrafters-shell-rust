# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, Gemini CLI, and others) when working with code in this repository.

## What this is

A POSIX-like shell (CodeCrafters "Build Your Own Shell" challenge), written in Rust. It's a REPL that
tokenizes input, parses it into an AST, and executes builtins/external programs/pipelines with
redirection support.

## Commands

```sh
cargo build                       # dev build
cargo build --release             # matches .codecrafters/compile.sh exactly
cargo run                         # run the shell interactively

cargo nextest run                 # run the ENTIRE test suite (unit + integration) — use this, not `cargo test`
cargo nextest run --lib           # unit tests only (src/**/*.rs #[cfg(test)] blocks)
cargo nextest run -E 'test(cli_)' # integration/black-box tests only (tests/cli_*.rs)
cargo nextest run <substring>     # run tests whose name contains <substring>, e.g.:
cargo nextest run cd_expands_home

cargo fmt                         # format
cargo clippy                      # lint
```

`cargo nextest` must be installed (`cargo install cargo-nextest --locked`) — it isn't a default cargo
subcommand. Nextest runs every test in its own OS process, which matters here: several tests call
`env::set_current_dir`/`env::set_var("PATH", ..)` or spawn the real compiled binary, and process-per-test
isolation is what makes that safe without manual serialization.

CodeCrafters submission flow (`git commit` + `git push origin master`) triggers `.codecrafters/compile.sh`
→ `.codecrafters/run.sh` remotely; don't rely on any local-only tooling in those scripts.

## Architecture

**Crate layout**: `src/lib.rs` re-exports every module (`commands`, `editor`, `error`, `parser`, `state`,
`utils`); `src/main.rs` is a thin REPL loop that only consumes the lib. This split exists specifically so
integration tests can `use codecrafters_shell::...` and spawn the real binary via
`env!("CARGO_BIN_EXE_codecrafters-shell")`. Keep new code in the lib, not in `main.rs`.

**Execution pipeline**, one line of input at a time:

1. `parser::Lexer::tokenizer` — a hand-rolled character state machine (`Normal` / `SingleQuote` /
   `DoubleQuote` / `Escape`), no parser-combinator crate. Produces `Vec<Token>`. Quoting/escaping rules are
   POSIX-ish but not identical to bash (see Known gaps).
2. `parser::Parser::parser` — hand-rolled recursive descent over the tokens → `ASTNode::Simple` or
   `ASTNode::Pipeline`. It only understands pipelines of simple commands today; `Token::And`/`Or`/
   `Background` exist but nothing consumes them (see Known gaps).
3. `commands::execute_ast` → `Command::resolve` picks `Builtin` vs `External` (PATH lookup lives in
   `utils::path`). Builtins are one function per file under `commands/executors/`, each shaped
   `fn(&ParsedCommand, &mut dyn Read, &mut dyn Write, ...) -> Result<()>` so the same function works
   whether stdio is a real terminal, a redirected file, or a pipeline buffer.
4. Redirection: `utils::redirection::ResolvedRedirections` opens real files per `Descriptor`
   (`Stdin`/`Stdout`/`Stderr`) + `RedirectionType` (`Input`/`Output`/`Append`), applied uniformly to
   builtins (`Command::execute`) and external commands (`commands/external.rs`, via
   `std::process::Command`).
5. Pipelines (`commands/executors/pipeline.rs::Pipeline`): the tricky part is that builtins have no OS-level
   stdout to pipe, so a non-last builtin's output is captured into an in-memory `Vec<u8>` and manually
   written into the next stage's stdin; external→external stages use real OS pipes (`Stdio::piped()`).

**Editor** (`editor/`): a thin wrapper around the `rustyline` crate, which does the actual line-editing.
`EditorHelper` implements only `Completer` (tab-completion for builtins / PATH executables / filesystem
paths, including bash-style double-tab-to-list-all); `Highlighter`/`Validator`/`Hinter` are no-ops.

**State** (`state.rs`): in-memory `history: Vec<String>` plus `HISTFILE` load-on-start/write-on-exit,
driven from `main.rs`, and the `history -r/-w/-a` builtin flags.

## Known gaps (don't silently "fix" these — flag and ask; tests can document current behavior instead of asserting a fix)

- `&&`, `||`, and background `&` are tokenized but the parser has no And/Or/Background handling — anything
  after the first pipeline segment on a line is silently dropped, no error.
- `exit <code>` ignores the code entirely; the process always exits 0.
- If a pipeline's mid-chain command fails to resolve (unknown command), earlier stages' spawned children
  are never `wait()`'d (`Pipeline::run` returns early via `?` before reaching `wait_for_children`).

## Testing — this project is developed test-first (TDD)

Workflow: tests are written before the implementation exists. Write tests that encode the **intended**
correct behavior precisely (exact stdout/stderr strings and exit codes — not just "doesn't crash"), so a
missing or wrong implementation fails clearly and a correct one passes unambiguously. Never weaken an
assertion just to make a not-yet-implemented feature pass.

**Two tiers — put each test in the right one:**

- `src/**/*.rs`, `#[cfg(test)] mod tests { ... }` blocks beside the code they test — for pure logic and
  anything that needs access to private items (e.g. `EditorHelper`'s private `find_*` methods). These call
  Rust functions directly, in-process, no subprocess.
- `tests/cli_*.rs` — black-box tests that spawn the actual compiled binary and drive it like a real user
  would (pipe a script into stdin, assert on captured stdout/stderr/exit status). This is the only tier
  that exercises `main.rs`'s REPL loop, real process spawning/piping, and `HISTFILE` load/save together.
  Use this tier for anything user-facing: a new builtin, a new redirection form, pipeline behavior.

**Sandboxing is mandatory, not optional** — a test must never touch the real filesystem, `$PATH`, `$HOME`,
or cwd outside its own throwaway directory:

- Unit tests: use `tempfile::TempDir` for anything file-based; if a test needs to mutate `PATH`/cwd
  (`env::set_var`, `env::set_current_dir`), that's fine under nextest's process-per-test model, but still
  scope every path inside a `TempDir`.
- CLI tests: use the `Sandbox` harness in `tests/support/mod.rs` (`mod support;` at the top of the file).
  `Sandbox::new()` gives you an isolated `work_dir`/`home_dir`/`bin_dir`/`histfile` inside a `TempDir`;
  `sandbox.run(script)` spawns the real binary with `env_clear()` and `PATH`/`HOME`/`HISTFILE`/cwd pointed
  only inside that sandbox — real system binaries are never on `PATH`. If a test needs an "external
  command" fixture, add a tiny `#!/bin/sh` script to `Sandbox::install_fixture_bins()` rather than
  depending on whatever happens to be installed on the host.
- Gotcha: `support::stdout()` strips the `"$ "` prompt that `rustyline` writes to real stdout on every
  `readline()` call (even for piped/non-tty input) — always assert through `support::stdout()`/`stderr()`,
  never on `Output.stdout` directly, or prompt noise will corrupt string-equality assertions.
- Any feature that's about *terminal interaction itself* (tab completion, bell, live redraw, anything
  gated on the input actually being a tty) cannot be tested through `sandbox.run()` — `rustyline` only
  engages its raw, key-by-key input handling when stdin is a real tty, so a piped script never triggers
  it at all. Use `sandbox.spawn_pty()` instead (see `tests/cli_tab_completion.rs` for the pattern), which
  returns a `rexpect::session::PtySession` spawned in the same sandboxed dirs/env but attached to a real
  pseudo-terminal. Send raw bytes with `.send()`/`.send_control()` + `.flush()`, assert on output with
  `.exp_string()`/`.exp_char()`/`.exp_regex()`. `TERM` is set to `xterm-256color` for these (not `dumb`,
  which disables interactive editing) — don't reuse `sandbox.run()`'s `TERM=dumb` assumption here.

**Coverage expectations per feature**: cover the happy path AND the error path (bad redirect target,
unknown command, syntax error, missing file) and assert the shell reports the error *and keeps running*
(feed a following command in the same script and assert it still executes) rather than just checking it
doesn't panic.
