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
