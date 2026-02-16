#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Display the prompt and ensure it appears immediately before waiting for input
    print!("$ ");
    io::stdout().flush().unwrap();

    // Read user input and trim trailing whitespace
    let mut user_command = String::new();
    io::stdin().read_line(&mut user_command).unwrap();
    user_command = user_command.trim_end().to_string();
    println!("{}: command not found", user_command);
}