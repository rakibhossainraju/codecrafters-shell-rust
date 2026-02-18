use std::error::Error;
use std::io::{self, Write};
use std::process::ExitStatus;

pub fn print_initial_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

pub fn print_command_not_found(command: &str) {
    eprintln!("{}: not found", command);
}

pub fn print_filed_to_execute(command: &str, err: impl Error) {
    eprintln!("Failed to execute command '{}': {}", command, err);
}

pub fn print_exit_with_status(command: &str, status: ExitStatus) {
    eprintln!("Command '{}' exited with status: {}", command, status);
}

pub fn read_user_command() -> String {
    let mut command = String::new();
    io::stdin().read_line(&mut command).expect("Failed to read line");
    command.trim().to_string()
}

pub fn parse_command(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current_arg = String::new();
    let mut chars = input.chars();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        if !in_single_quote && !in_double_quote {
            // STATE 1: Outside any quotes
            match c {
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                '\\' => {
                    if let Some(escaped_char) = chars.next() {
                        current_arg.push(escaped_char);
                    }
                }
                _ if c.is_whitespace() => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                },
                _ => current_arg.push(c),
            }
        } else if in_single_quote {
            // STATE 2: Inside single quotes
            // POSIX rule: EVERYTHING is literal in single quotes. No escaping allowed.
            match c {
                '\'' => in_single_quote = true,
                _ => current_arg.push(c),
            }
        } else if in_double_quote {
            // STATE 3: Inside double quotes
            match c {
                '"' => in_double_quote = true,
                '\\' => {
                    // Inside double quotes, we usually only escape " and \
                    if let Some(escaped_char) = chars.next() {
                        if escaped_char == '"' || escaped_char == '\\' {
                            current_arg.push(escaped_char);
                        } else {
                            // If it's something else like \n, keep the backslash and the char
                            current_arg.push('\\');
                            current_arg.push(escaped_char);
                        }
                    }
                }
                _ => current_arg.push(c),
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}