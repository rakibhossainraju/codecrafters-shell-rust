use std::cmp::PartialEq;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BuiltinCommands {
    Clear,
    Exit,
    Echo,
    Help,
    Type,
    Pwd,
    Cd,
    History,
}
pub const BUILTIN_COMMANDS: &[(&str, BuiltinCommands)] = &[
    ("clear", BuiltinCommands::Clear),
    ("exit", BuiltinCommands::Exit),
    ("echo", BuiltinCommands::Echo),
    ("help", BuiltinCommands::Help),
    ("type", BuiltinCommands::Type),
    ("pwd", BuiltinCommands::Pwd),
    ("cd", BuiltinCommands::Cd),
    ("history", BuiltinCommands::History),
];

impl BuiltinCommands {
    pub fn from_str(s: &str) -> Option<Self> {
        for (name, cmd) in BUILTIN_COMMANDS {
            if *name == s {
                return Some(cmd.clone());
            }
        }
        None
    }

    pub fn is_builtin_command(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
}

impl Display for BuiltinCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, cmd) in BUILTIN_COMMANDS {
            if *cmd == *self {
                return write!(f, "{}", name);
            }
        }
        Err(std::fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_every_registered_builtin_name() {
        for (name, cmd) in BUILTIN_COMMANDS {
            assert_eq!(BuiltinCommands::from_str(name), Some(*cmd));
            assert!(BuiltinCommands::is_builtin_command(name));
        }
    }

    #[test]
    fn unknown_command_is_not_a_builtin() {
        assert_eq!(BuiltinCommands::from_str("not-a-real-command"), None);
        assert!(!BuiltinCommands::is_builtin_command("ls"));
    }

    #[test]
    fn display_round_trips_through_the_lookup_table() {
        for (name, cmd) in BUILTIN_COMMANDS {
            assert_eq!(cmd.to_string(), *name);
        }
    }

    #[test]
    fn lookup_is_case_sensitive() {
        assert_eq!(BuiltinCommands::from_str("ECHO"), None);
        assert_eq!(BuiltinCommands::from_str("Echo"), None);
    }
}
