use crate::{
    commands::{BuiltinCommands, Command, INTERNAL_BUILTIN_MARKER, INTERNAL_PIPELINE_MARKER},
    error::{Result, ShellError},
    parser::{ASTNode, ParsedCommand},
    state::ShellState,
    utils::redirection::ResolvedReDirections,
};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command as OsCommand, Stdio};

use super::pipeline_transfer::encode_pipeline;

pub fn execute_background(ast: ASTNode, state: &mut ShellState) -> Result<()> {
    match ast {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = Command::resolve(parsed_cmd)?;

            match cmd {
                Command::Builtin(BuiltinCommands::Exit, _) => {
                    return Err(ShellError::ExitOut);
                }
                Command::External(external_cmd) => {
                    // `Some(Stdio::null())` as the default: a backgrounded
                    // command must never be able to steal keystrokes meant
                    // for the interactive shell's next prompt. An explicit
                    // `<` redirect on the command still takes precedence
                    // (see `ExternalCommand::as_os_command`).
                    let child = external_cmd.spawn(Some(Stdio::null()), None)?;
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
            let child = spawn_pipeline_job(&cmds)?;
            let cmd_str = cmds
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" | ");
            state.jobs.push_job(child, cmd_str);
        }
        ASTNode::Background(_) => {
            unreachable!("the parser never nests Background inside Background")
        }
        ASTNode::And(_, _) => {
            // Backgrounding a whole `&&` chain (`a && b &`) would need its
            // own re-exec + transfer scheme, same idea as
            // `spawn_pipeline_job` but for an And-tree instead of a flat
            // command list. Not built yet -- fail clearly rather than
            // silently running it in the foreground or panicking.
            return Err(ShellError::SyntaxError(
                "backgrounding a `&&` chain is not yet supported".to_string(),
            ));
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
    // Same stdin isolation as the external-command path above: default to
    // `/dev/null` so this builtin can't steal input meant for the
    // interactive shell, unless it has its own explicit `<` redirect.
    match resolved.stdin {
        Some(stdin) => cmd.stdin(Stdio::from(stdin)),
        None => cmd.stdin(Stdio::null()),
    };
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

/// Backgrounds an entire pipeline (`cmd1 | cmd2 &`) the same way a single
/// builtin is backgrounded, just at a coarser grain: re-exec this shell
/// binary as a child process, but this time tell it to run the *whole*
/// pipeline (via the existing, unmodified `execute_pipeline`) and then exit.
/// One child, tracked as one job.
///
/// This has to happen at pipeline granularity rather than per-stage: a
/// pipeline's builtin stages have no OS process of their own either (see
/// `Pipeline::execute_builtin` — their output is captured into an in-memory
/// buffer), so there's no way to make just the builtin part of a pipeline
/// async without giving the *whole* pipeline a process boundary.
///
/// Unlike `spawn_builtin_job`, redirects are **not** resolved here — each
/// stage's redirects travel with it inside the encoded payload and are
/// resolved by `execute_pipeline` itself inside the child, exactly as they
/// are for a foreground pipeline. The only thing set here is the child
/// process's own stdin, forced to `/dev/null` for the same reason as
/// everywhere else in this file: a stage with its own explicit `<` redirect
/// still wins, since `execute_pipeline` resolves that before ever falling
/// back to inherited stdin.
fn spawn_pipeline_job(cmds: &[ParsedCommand]) -> Result<Child> {
    let current_exe = std::env::current_exe().map_err(|source| ShellError::ExecutionError {
        command: "pipeline".to_string(),
        source,
    })?;

    let mut cmd = OsCommand::new(current_exe);
    cmd.arg0("pipeline");
    cmd.arg(INTERNAL_PIPELINE_MARKER);
    cmd.arg(encode_pipeline(cmds));
    cmd.stdin(Stdio::null());

    cmd.spawn().map_err(|source| ShellError::ExecutionError {
        command: "pipeline".to_string(),
        source,
    })
}
