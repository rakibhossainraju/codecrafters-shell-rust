use std::io::{self, Write};
use std::fs::File;
use crate::parser::{Descriptor, ParsedCommand, RedirectionType};

pub fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

pub fn read_user_command() -> String {
    let mut command = String::new();
    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");
    command.trim().to_string()
}

pub fn get_stdout(parsed_cmd: &ParsedCommand) -> Box<dyn Write> {
    // Default to terminal screen
    let mut final_writer: Box<dyn Write> = Box::new(io::stdout());

    for redirect in &parsed_cmd.redirects {
        // We only care about Stdout for this specific writer
        if redirect.redirection_type == RedirectionType::Output
            && redirect.descriptor == Descriptor::Stdout
        {
            // Open the file (this creates/truncates it, satisfying POSIX rules)
            match File::create(&redirect.file) {
                Ok(file) => {
                    // Overwrite our final_writer. If there's another redirect after this,
                    // this file will be dropped (and safely closed), and the next one takes over!
                    final_writer = Box::new(file);
                }
                Err(e) => {
                    eprintln!("shell: {}: {}", redirect.file, e);
                    // In a real shell, if one file fails (e.g., permission denied),
                    // the command aborts entirely. For now, returning stdout or
                    // breaking is an okay fallback.
                    return Box::new(io::stdout());
                }
            }
        }
    }

    final_writer
}