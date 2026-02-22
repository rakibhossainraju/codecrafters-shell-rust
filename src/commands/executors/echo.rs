use crate::parser::ParsedCommand;

/// Execute the echo builtin command
pub fn execute_echo(parsed_cmd: ParsedCommand) {
    if parsed_cmd.args.is_empty() {
        println!();
    } else {
        println!("{}", parsed_cmd.args.join(" "));
    }
}
