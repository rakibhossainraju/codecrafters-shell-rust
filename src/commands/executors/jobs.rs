use crate::{error::{Result, ShellError}, state::ShellState};
use std::io::{Read, Write};

pub fn execute_jobs(_stdin: &mut dyn Read, stdout: &mut dyn Write, state: &mut ShellState) -> Result<()> {
    state.jobs.print_jobs();
    // Jobs are not implemented yet;
    Ok(())
}
