use crate::error::Result;
use crate::parser::ParsedCommand;
use std::io::Write;

/// Execute the echo builtin command
pub fn execute_echo(parsed_cmd: &ParsedCommand, stdout: &mut dyn Write) -> Result<()> {
    if parsed_cmd.args.is_empty() {
        writeln!(stdout)?;
    } else {
        writeln!(stdout, "{}", parsed_cmd.args.join(" "))?;
    }
    Ok(())
}
