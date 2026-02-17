use std::io::{self, Write};

pub fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

pub fn print_command_not_found(command: &str) {
    eprintln!("{}: not found", command);
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