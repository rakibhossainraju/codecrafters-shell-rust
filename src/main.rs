mod commands;
mod utils;

use commands::{BuiltinCommands, Command};
use crate::utils::{print_initial_prompt, read_user_command};

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



