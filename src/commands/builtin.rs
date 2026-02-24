use std::fmt::Display;

pub const BUILTIN_COMMANDS: &[&str] = &["clear", "exit", "echo", "help", "type", "pwd", "cd"];

#[derive(Clone, Copy)]
pub enum BuiltinCommands {
    Clear,
    Exit,
    Echo,
    Help,
    Type,
    Pwd,
    Cd,
}

impl BuiltinCommands {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "clear" | "c"    => Some(BuiltinCommands::Clear),
            "exit" | "exit;" => Some(BuiltinCommands::Exit),
            "echo"           => Some(BuiltinCommands::Echo),
            "help"           => Some(BuiltinCommands::Help),
            "type"           => Some(BuiltinCommands::Type),
            "pwd"            => Some(BuiltinCommands::Pwd),
            "cd"             => Some(BuiltinCommands::Cd),
            _                => None,
        }
    }

    pub fn is_builtin_command(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
}

impl Display for BuiltinCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinCommands::Cd => write!(f, "cd"),
            BuiltinCommands::Clear => write!(f, "clear"),
            BuiltinCommands::Exit => write!(f, "exit"),
            BuiltinCommands::Echo => write!(f, "echo"),
            BuiltinCommands::Help => write!(f, "help"),
            BuiltinCommands::Type => write!(f, "type"),
            BuiltinCommands::Pwd => write!(f, "pwd"),
        }
    }
}
