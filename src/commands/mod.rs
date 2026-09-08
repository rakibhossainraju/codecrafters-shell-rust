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

pub fn execute_ast(ast: ASTNodes, state: &mut ShellState) -> Result<()> {
    for ast_node in ast {
        ast_executor(ast_node, state)?;
    }
    Ok(())
}

fn ast_executor(ast_node: ASTNode, state: &mut ShellState) -> Result<()> {
    match ast_node {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = Command::resolve(parsed_cmd)?;

            // Check for exit before executing (to break the loop)
            if matches!(cmd, Command::Builtin(BuiltinCommands::Exit, _)) {
                return Err(ShellError::ExitOut);
            }

            // Execute the command with remaining arguments
            cmd.execute(None, None, state)?;
            // If we get here, the command executed successfully
            Ok(())
        }
        ASTNode::Pipeline(cmds) => {
            execute_pipeline(cmds, state)?;
            Ok(())
        }
        ASTNode::Background(ast) => {
            execute_background(*ast, state)?;
            Ok(())
        }
    }
}
