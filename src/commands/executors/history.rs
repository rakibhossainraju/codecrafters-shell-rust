use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::state::ShellState;
use std::io::{Read, Write};

const DEFAULT_HISTORY_SIZE: usize = 10;

pub fn execute_history(
    parsed_cmd: &ParsedCommand,
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    state: &mut ShellState,
) -> Result<()> {
    if !parsed_cmd.args.is_empty() && parsed_cmd.args[0] == "-r" {
        let filename = parsed_cmd.args.get(1).ok_or_else(|| {
            ShellError::InvalidArgument("Expected filename after -r".to_string())
        })?;
        state.load_history(filename)?;
        return Ok(());
    }

    let size = if parsed_cmd.args.is_empty() {
        DEFAULT_HISTORY_SIZE.min(state.history.len())
    } else {
        if parsed_cmd.args.len() > 1 {
            return Err(ShellError::TooManyArguments);
        }
        parsed_cmd.args[0]
            .parse::<usize>()
            .map(|n| n.min(state.history.len()))
            .map_err(|_| ShellError::InvalidArgument(parsed_cmd.args[0].to_string()))?
    };

    let start = state.history.len().saturating_sub(size);
    for (i, cmd) in state.history.iter().enumerate().skip(start) {
        writeln!(stdout, "{:>5}  {}", i + 1, cmd).map_err(ShellError::IoError)?;
    }
    Ok(())
}
