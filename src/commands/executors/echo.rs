use crate::parser::ParsedCommand;
use crate::error::Result;

/// Execute the echo builtin command
pub fn execute_echo(parsed_cmd: &ParsedCommand) -> Result<()> {
    if parsed_cmd.args.is_empty() {
        println!();
    } else {
        println!("{}", parsed_cmd.args.join(" "));
    }
    Ok(())
}
