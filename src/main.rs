mod commands;
mod utils;
mod parser;

use commands::{BuiltinCommands, Command};

fn main() {
    loop {
        // Display the prompt and ensure it appears immediately before waiting for input
        utils::print_initial_prompt();

        // Read user input and trim trailing whitespace
        let user_input = utils::read_user_command();
        let args = parser::parse_command(&user_input);

        if let Some(command_str) = args.get(0) {
            let cmd: Command = command_str.as_str().into();

            // Check for exit before executing (to break the loop)
            if matches!(cmd, Command::Builtin(BuiltinCommands::Exit)) {
                break;
            }

            // Execute the command with remaining arguments
            cmd.execute(args);
        }
    }
}
