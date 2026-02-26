use rustyline::error::ReadlineError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("{0}: not found")]
    CommandNotFound(String),

    #[error("syntax error: {0}")]
    SyntaxError(String),

    #[error("failed to execute command '{command}': {source}")]
    ExecutionError {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("command '{command}' exited with status: {status}")]
    ExitWithStatus {
        command: String,
        status: std::process::ExitStatus,
    },

    #[error("failed to wait for command '{0}'")]
    WaitError(String),

    #[error("io error: {0}")]
    IoError(#[from] io::Error),

    #[error("cd: {0}: No such file or directory")]
    CdError(String),

    #[error("readline error: {0}")]
    Readline(#[from] ReadlineError),

    #[error("exit")]
    ExitOut,
}

pub type Result<T> = std::result::Result<T, ShellError>;
