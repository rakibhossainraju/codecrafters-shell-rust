mod builtin;
mod command;
mod executors;
mod external;

use crate::commands::executors::background::execute_background;
use crate::commands::executors::pipeline::execute_pipeline;
use crate::error::{Result, ShellError};
use crate::parser::{ASTNode, ASTNodes, ParsedCommand};
use crate::state::ShellState;
pub use builtin::*;
pub use command::*;
pub use external::*;

/// Hidden argv marker that tells `main` to skip the interactive REPL and
/// instead run exactly one builtin non-interactively via
/// [`run_internal_builtin`], then exit. This is how backgrounding a builtin
/// (`echo hi &`) works: since a builtin has no OS process of its own to hand
/// off to the kernel the way an external command's `Child` does, the shell
/// re-execs its own binary as a child process with this marker plus the
/// builtin's name/args, and treats the resulting `Child` exactly like any
/// external command's job. See `executors::background::spawn_builtin_job`.
pub const INTERNAL_BUILTIN_MARKER: &str = "--__shell-internal-run-builtin";

/// Entry point for a re-exec'd child spawned by `spawn_builtin_job`. `args`
/// is everything after `INTERNAL_BUILTIN_MARKER` on argv: the builtin's name
/// followed by its own arguments. Redirections were already resolved by the
/// parent before spawning (see `spawn_builtin_job`), so this process's
/// stdin/stdout/stderr are already wired to the right files/pipes and are
/// used as-is here.
///
/// Note this child starts from a *fresh* `ShellState` — unlike a real
/// `fork()`, re-exec gives it a brand-new process image, not a copy-on-write
/// snapshot of the parent's memory. History is approximated by reloading
/// `HISTFILE`; the parent's in-memory job list has no equivalent and is
/// simply unavailable (matches real shells: a backgrounded builtin runs in a
/// subshell and can't observe or mutate the parent's state either).
pub fn run_internal_builtin(args: &[String]) -> ! {
    let mut state = ShellState::new();
    if let Ok(histfile) = std::env::var("HISTFILE") {
        let _ = state.history.load_history(&histfile);
    }

    let exit_code = match args.split_first() {
        Some((name, rest)) => match name.parse::<BuiltinCommands>() {
            Ok(builtin) => {
                let parsed_cmd = ParsedCommand {
                    cmd: name.clone(),
                    args: rest.to_vec(),
                    redirects: Vec::new(),
                };
                match Command::Builtin(builtin, parsed_cmd).execute(None, None, &mut state) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("{}", e);
                        1
                    }
                }
            }
            Err(_) => {
                eprintln!("{}: not a builtin", name);
                1
            }
        },
        None => 1,
    };

    std::process::exit(exit_code);
}

/// Hidden argv marker for backgrounding a whole pipeline (`cmd1 | cmd2 &`),
/// the same idea as [`INTERNAL_BUILTIN_MARKER`] but at pipeline granularity:
/// a pipeline's builtin stages have no OS process of their own either (they
/// run in-process, capturing output into memory — see `Pipeline::execute_builtin`),
/// so there's no way to make part of a pipeline async without giving the
/// *whole* pipeline a process boundary. One re-exec'd child runs the entire
/// pipeline via the existing, unmodified `execute_pipeline`, and is tracked
/// as one job. See `executors::background::spawn_pipeline_job`.
pub const INTERNAL_PIPELINE_MARKER: &str = "--__shell-internal-run-pipeline";

/// Entry point for a re-exec'd child spawned by `spawn_pipeline_job`. `args`
/// is a single element: the whole pipeline encoded by
/// `executors::pipeline_transfer::encode_pipeline`. Each stage's own
/// redirects travel inside that payload and are resolved by
/// `execute_pipeline` itself, exactly as they are for a foreground pipeline
/// — nothing about that logic changes, only where it runs.
pub fn run_internal_pipeline(args: &[String]) -> ! {
    use crate::commands::executors::pipeline_transfer::decode_pipeline;

    let mut state = ShellState::new();
    if let Ok(histfile) = std::env::var("HISTFILE") {
        let _ = state.history.load_history(&histfile);
    }

    let exit_code = match args.first().map(|payload| decode_pipeline(payload)) {
        Some(Ok(cmds)) => match execute_pipeline(cmds, &mut state) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        Some(Err(e)) => {
            eprintln!("{}", e);
            1
        }
        None => 1,
    };

    std::process::exit(exit_code);
}

/// Hidden argv marker for backgrounding a whole `&&` chain (`a && b &`), the
/// same idea as [`INTERNAL_PIPELINE_MARKER`] one level up: a chain's leaves
/// are `Simple`/`Pipeline` nodes with no OS process boundary between them on
/// their own, so short-circuiting has to happen somewhere, and giving the
/// *whole chain* one process is what lets it reuse everything that already
/// exists (`ast_executor`'s short-circuit logic, `execute_pipeline`) instead
/// of re-implementing `&&` semantics a second time just for the backgrounded
/// case. See `executors::background::spawn_and_chain_job`.
pub const INTERNAL_AND_CHAIN_MARKER: &str = "--__shell-internal-run-and-chain";

/// Entry point for a re-exec'd child spawned by `spawn_and_chain_job`.
/// `args` is a single element: the chain's leaves (each a command or a
/// whole pipeline), encoded by
/// `executors::pipeline_transfer::encode_and_chain`. Runs them in order,
/// short-circuiting on the first failure, via the same `ast_executor` the
/// foreground REPL loop uses — so the short-circuit rule only exists in one
/// place.
pub fn run_internal_and_chain(args: &[String]) -> ! {
    use crate::commands::executors::pipeline_transfer::decode_and_chain;

    let mut state = ShellState::new();
    if let Ok(histfile) = std::env::var("HISTFILE") {
        let _ = state.history.load_history(&histfile);
    }

    let exit_code = match args.first().map(|payload| decode_and_chain(payload)) {
        Some(Ok(chains)) => run_and_chain(chains, &mut state),
        Some(Err(e)) => {
            eprintln!("{}", e);
            1
        }
        None => 1,
    };

    std::process::exit(exit_code);
}

/// Runs each leaf of a decoded `&&` chain in order via `ast_executor`,
/// stopping at the first failure. `ast_executor` only ever returns `Err` for
/// `ShellError::ExitOut` (see its doc comment) -- if a leaf is literally
/// `exit`, that should end *this* re-exec'd subshell, not propagate
/// anywhere, so it's treated as a clean stop here rather than re-raised.
fn run_and_chain(chains: Vec<Vec<ParsedCommand>>, state: &mut ShellState) -> i32 {
    let mut succeeded = true;
    for cmds in chains {
        if !succeeded {
            break;
        }
        let node = match cmds.len() {
            1 => ASTNode::Simple(cmds.into_iter().next().expect("len checked above")),
            _ => ASTNode::Pipeline(cmds),
        };
        succeeded = match ast_executor(node, state) {
            Ok(ok) => ok,
            Err(ShellError::ExitOut) => return 0,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        };
    }
    i32::from(!succeeded)
}

pub fn execute_ast(ast: ASTNodes, state: &mut ShellState) -> Result<()> {
    for ast_node in ast {
        ast_executor(ast_node, state)?;
    }
    Ok(())
}

/// Executes one top-level AST item and reports whether it succeeded (exit
/// status 0 for an external command, `Ok(())` for a builtin).
///
/// Only `ShellError::ExitOut` propagates as a hard `Err` out of here — every
/// other execution failure (command not found, a builtin's own error, a
/// nonzero exit code) is printed immediately, exactly as before this
/// function returned `Result<bool>`, and folded into `Ok(false)` rather than
/// aborting. That's the part `&&` actually needs: the left side failing
/// has to be visible (printed, and skip the right side) without also
/// preventing an unrelated sibling item on the same line from running.
fn ast_executor(ast_node: ASTNode, state: &mut ShellState) -> Result<bool> {
    match ast_node {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = match Command::resolve(parsed_cmd) {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(false);
                }
            };

            // Check for exit before executing (to break the loop)
            if matches!(cmd, Command::Builtin(BuiltinCommands::Exit, _)) {
                return Err(ShellError::ExitOut);
            }

            Ok(run_simple_command(&cmd, state))
        }
        ASTNode::Pipeline(cmds) => match execute_pipeline(cmds, state) {
            Ok(()) => Ok(true),
            Err(e) => {
                eprintln!("{}", e);
                Ok(false)
            }
        },
        ASTNode::Background(ast) => match execute_background(*ast, state) {
            Ok(()) => Ok(true),
            Err(ShellError::ExitOut) => Err(ShellError::ExitOut),
            Err(e) => {
                eprintln!("{}", e);
                Ok(false)
            }
        },
        ASTNode::And(left, right) => {
            if ast_executor(*left, state)? {
                ast_executor(*right, state)
            } else {
                Ok(false)
            }
        }
    }
}

/// Runs a resolved, non-`exit` command and reports whether it succeeded.
/// Builtins already use `Result` for exactly this (`Ok` = success, `Err` =
/// failure), so that case just prints and folds like everywhere else in
/// `ast_executor`. Externals need their *real* exit code, though —
/// `Command::execute`'s `Result<()>` only signals whether the OS could
/// spawn and wait for the process at all, not what it actually exited
/// with (see `executors::external::execute_external_command`) — so this
/// spawns and waits directly instead of going through it.
fn run_simple_command(cmd: &Command, state: &mut ShellState) -> bool {
    match cmd {
        Command::Builtin(_, _) => match cmd.execute(None, None, state) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        },
        Command::External(external_cmd) => match external_cmd.spawn(None, None) {
            Ok(mut child) => match child.wait() {
                Ok(status) => status.success(),
                Err(_) => {
                    eprintln!(
                        "{}",
                        ShellError::WaitError(external_cmd.parsed_cmd.cmd.clone())
                    );
                    false
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        },
    }
}
