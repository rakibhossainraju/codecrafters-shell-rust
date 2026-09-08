use crate::{
    error::{Result, ShellError},
    state::ShellState,
};
use std::io::{Read, Write};

pub fn execute_jobs(
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    state: &mut ShellState,
) -> Result<()> {
    let jobs_list = state.jobs.format_jobs();

    write!(stdout, "{}", jobs_list)?;

    Ok(())
}
