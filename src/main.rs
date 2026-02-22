mod commands;
mod parser;
mod utils;

use commands::{BuiltinCommands, Command};
use crate::parser::{Lexer, Parser};

fn main() {
    loop {
        // Display the prompt and ensure it appears immediately before waiting for input
        utils::print_initial_prompt();

        // Read user input and trim trailing whitespace
        let user_input = utils::read_user_command();
        let tokens = Lexer::tokenizer(&user_input);
        let parsed_cmd = Parser::parser(tokens);
        
        let cmd: Command = parsed_cmd.cmd.as_str().into();
        // Check for exit before executing (to break the loop)
        if matches!(cmd, Command::Builtin(BuiltinCommands::Exit)) {
            break;
        }

        // Execute the command with remaining arguments
        cmd.execute(parsed_cmd);
    }
}
