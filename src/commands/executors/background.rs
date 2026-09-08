use crate::{
    commands::{BuiltinCommands, Command, INTERNAL_BUILTIN_MARKER, executors::pipeline::execute_pipeline},
    error::{Result, ShellError},
    parser::{ASTNode, ParsedCommand},
    state::ShellState,
    utils::redirection::ResolvedReDirections,
};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command as OsCommand, Stdio};

pub fn execute_background(ast: ASTNode, state: &mut ShellState) -> Result<()> {
    match ast {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = Command::resolve(parsed_cmd)?;

            match cmd {
                Command::Builtin(BuiltinCommands::Exit, _) => {
                    return Err(ShellError::ExitOut);
                }
                Command::External(external_cmd) => {
                    let child = external_cmd.spawn(None, None)?;
                    let cmd_str = external_cmd.parsed_cmd.to_string();
                    state.jobs.push_job(child, cmd_str);
                }
                Command::Builtin(_, parsed_cmd) => {
                    let child = spawn_builtin_job(&parsed_cmd)?;
                    let cmd_str = parsed_cmd.to_string();
                    state.jobs.push_job(child, cmd_str);
                }
            }
        }
        ASTNode::Pipeline(cmds) => {
            execute_pipeline(cmds, state)?;
        }
        ASTNode::Background(_) => {
            unreachable!("the parser never nests Background inside Background")
        }
    }
    Ok(())
}

/// Backgrounds a builtin by re-executing this same shell binary as a child
/// process in a hidden "run one builtin and exit" mode (see
/// `commands::run_internal_builtin`), rather than a real `fork()`. `std`
/// doesn't expose `fork()`, and a raw libc/nix fork would be genuinely
/// unsafe here since the process is already multi-threaded (rustyline's
/// terminal handling) — any lock held by another thread at fork time would
/// stay locked forever in the child. Re-exec sidesteps all of that and
/// reuses the exact `std::process::Child`-based job model already built for
/// external commands.
///
/// Redirections are resolved here, in the parent, exactly once, and handed
/// to the child as real file descriptors — the child never needs to know
/// about `parsed_cmd.redirects` at all, it just inherits already-correct
/// stdio.
fn spawn_builtin_job(parsed_cmd: &ParsedCommand) -> Result<Child> {
    let current_exe = std::env::current_exe().map_err(|source| ShellError::ExecutionError {
        command: parsed_cmd.cmd.clone(),
        source,
    })?;

    let mut cmd = OsCommand::new(current_exe);
    // Mirrors ExternalCommand::spawn: argv[0] shows the builtin's name
    // (e.g. "echo"), not the shell binary's own path.
    cmd.arg0(&parsed_cmd.cmd);
    cmd.arg(INTERNAL_BUILTIN_MARKER);
    cmd.arg(&parsed_cmd.cmd);
    cmd.args(&parsed_cmd.args);

    let resolved = ResolvedReDirections::resolve(parsed_cmd)?;
    if let Some(stdin) = resolved.stdin {
        cmd.stdin(Stdio::from(stdin));
    }
    if let Some(stdout) = resolved.stdout {
        cmd.stdout(Stdio::from(stdout));
    }
    if let Some(stderr) = resolved.stderr {
        cmd.stderr(Stdio::from(stderr));
    }

    cmd.spawn().map_err(|source| ShellError::ExecutionError {
        command: parsed_cmd.cmd.clone(),
        source,
    })
}
