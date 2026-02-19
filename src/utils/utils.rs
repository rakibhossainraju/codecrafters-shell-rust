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

pub fn print_failed_to_execute(command: &str, err: impl Error) {
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