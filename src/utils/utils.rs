use std::io::{self, Write};

pub fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}