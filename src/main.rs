mod commands;

#[allow(unused_imports)]
use std::io::{self, Write};
use commands::{Command, BuiltinCommands};

fn main() {
    loop {
        // Display the prompt and ensure it appears immediately before waiting for input
        print_initial_prompt();

        // Read user input and trim trailing whitespace
        let user_input = read_user_command();

        if let Some(command_str) = user_input.get(0) {
            let cmd: Command = command_str.as_str().into();

            // Check for exit before executing (to break the loop)
            if matches!(cmd, Command::Builtin(BuiltinCommands::Exit)) {
                break;
            }

            // Execute the command with remaining arguments
            cmd.execute(&user_input[1..]);
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
