use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::state::ShellState;
use std::io::{Read, Write};

const DEFAULT_HISTORY_SIZE: usize = 10;

enum HistoryAction {
    Display(usize),
    Read(String),
    Write(String),
    Append(String),
}

impl HistoryAction {
    fn from_args(args: &[String], current_history_len: usize) -> Result<Self> {
        if args.is_empty() {
            return Ok(HistoryAction::Display(
                DEFAULT_HISTORY_SIZE.min(current_history_len),
            ));
        }

        match args[0].as_str() {
            "-r" => {
                let filename = args.get(1).ok_or_else(|| {
                    ShellError::InvalidArgument("Expected filename after -r".to_string())
                })?;
                Ok(HistoryAction::Read(filename.clone()))
            }
            "-w" => {
                let filename = args.get(1).ok_or_else(|| {
                    ShellError::InvalidArgument("Expected filename after -w".to_string())
                })?;
                Ok(HistoryAction::Write(filename.clone()))
            }
            "-a" => {
                let filename = args.get(1).ok_or_else(|| {
                    ShellError::InvalidArgument("Expected filename after -a".to_string())
                })?;
                Ok(HistoryAction::Append(filename.clone()))
            }
            arg => {
                if args.len() > 1 {
                    return Err(ShellError::TooManyArguments);
                }
                let size = arg
                    .parse::<usize>()
                    .map(|n| n.min(current_history_len))
                    .map_err(|_| ShellError::InvalidArgument(arg.to_string()))?;
                Ok(HistoryAction::Display(size))
            }
        }
    }
}

pub fn execute_history(
    parsed_cmd: &ParsedCommand,
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    state: &mut ShellState,
) -> Result<()> {
    let action = HistoryAction::from_args(&parsed_cmd.args, state.history.len())?;

    match action {
        HistoryAction::Display(size) => {
            let start = state.history.len().saturating_sub(size);
            for (i, cmd) in state.history.iter().enumerate().skip(start) {
                writeln!(stdout, "{:>5}  {}", i + 1, cmd).map_err(ShellError::IoError)?;
            }
        }
        HistoryAction::Read(filename) => {
            state.load_history(&filename)?;
        }
        HistoryAction::Write(filename) => {
            state.write_history(&filename)?;
        }
        HistoryAction::Append(filename) => {
            state.append_history(&filename)?;
        }
    }
    Ok(())
}
