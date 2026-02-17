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

pub fn read_user_command() -> Vec<String> {
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command
        .trim_end()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
