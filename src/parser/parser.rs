
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
                // '>' => {
                //     if !current_arg.is_empty() {
                //         args.push(current_arg.clone());
                //         current_arg.clear();
                //     }
                //     args.push(">".to_string());
                // }
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
                '\'' => in_single_quote = false,
                _ => current_arg.push(c),
            }
        } else if in_double_quote {
            // STATE 3: Inside double quotes
            match c {
                '"' => in_double_quote = false,
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