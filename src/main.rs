#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Display the prompt and ensure it appears immediately before waiting for input
        print_initial_prompt();
        // Read user input and trim trailing whitespace
        let user_command = read_user_command();
        match user_command.as_str() {
            "exit" => break,
            _ => {}   
        }
        println!("{}: command not found", user_command);
    }
}

fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn read_user_command() -> String {
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command = command.trim_end().to_string();
    command
}