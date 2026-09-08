use rustyline::error::ReadlineError;
use std::io;
use thiserror::Error;

use crate::parser::Token;

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

    #[error("fc: too many arguments")]
    TooManyArguments,

    #[error("fc: invalid argument: {0}")]
    InvalidArgument(String),

    #[error("could not write to history file: {0}")]
    HistoryWriteError(io::Error),

    #[error("syntax error near unexpected token `{0}`")]
    ParserSyntaxError(Token),
}

pub type Result<T> = std::result::Result<T, ShellError>;
