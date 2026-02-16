use std::fmt::Display;

pub enum BuiltinCommands {
    Exit,
    Echo,
    Help,
    Type
}

impl BuiltinCommands {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(BuiltinCommands::Exit),
            "echo" => Some(BuiltinCommands::Echo),
            "help" => Some(BuiltinCommands::Help),
            "type" => Some(BuiltinCommands::Type),
            _ => None,
        }
    }

    pub fn is_builtin_command(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
}

impl Display for BuiltinCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinCommands::Exit => write!(f, "exit"),
            BuiltinCommands::Echo => write!(f, "echo"),
            BuiltinCommands::Help => write!(f, "help"),
            BuiltinCommands::Type => write!(f, "type"),
        }
    }
}
