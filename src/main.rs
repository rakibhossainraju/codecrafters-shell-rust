#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Display the prompt and ensure it appears immediately before waiting for input
        print_initial_prompt();
        // Read user input and trim trailing whitespace
        let user_input = read_user_command();
        match user_input.get(0) {
            Some(command) => {
                match command.as_str() {
                    "exit"  => break,
                    "echo" => {
                        execute_echo(&user_input[1..]);
                    }
                    _ => {
                        println!("{}: command not found", command);
                    }
                }
            }
            None => {
                println!("Invalid command.");
            }
        }
    }
}

fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn read_user_command() -> Vec<String> {
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command
        .trim_end()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

fn execute_echo(args: &[String]) {
    if args.len() == 0 {
        println!("No arguments supplied.");
    }
    let args = args.iter()
        .map(|s| s.to_string() + " ")
        .collect::<String>();
    println!("{}", args);
}