use std::error::Error;
use std::io::{self, Write};
use std::process::ExitStatus;

pub fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

pub fn print_command_not_found(command: &str) {
    eprintln!("{}: not found", command);
}

pub fn print_filed_to_execute(command: &str, err: impl Error) {
    eprintln!("Failed to execute command '{}': {}", command, err);
}

pub fn print_exit_with_status(command: &str, status: ExitStatus) {
    eprintln!("Command '{}' exited with status: {}", command, status);
}

pub fn read_user_command() -> String {
    let mut command = String::new();
    io::stdin().read_line(&mut command).expect("Failed to read line");
    command.trim().to_string()
}

pub fn parse_command(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();

    // Keep track of our current "state"
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in input.chars() {
        if c == '\'' && !in_double_quote {
            // Toggle the single quote state (ignore if inside double quotes)
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            // Toggle double quote state (ignore if inside single quotes)
            in_double_quote = !in_double_quote;
        } else if c.is_whitespace() && !in_single_quote && !in_double_quote {
            // If we hit a space AND we are not inside any quotes, the argument is done
            if !current_arg.is_empty() {
                args.push(current_arg.clone());
                current_arg.clear(); // Reset for the next argument
            }
        } else {
            // It's a normal character (or a space inside quotes), add it to the argument
            current_arg.push(c);
        }
    }
    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    args
}