use std::cmp::PartialEq;

use strum::{Display as StrumDisplay, EnumIter, EnumString};

#[derive(Clone, Copy, PartialEq, Debug, EnumIter, StrumDisplay, EnumString)]
pub enum BuiltinCommands {
    #[strum(to_string = "clear")]
    Clear,
    #[strum(to_string = "exit")]
    Exit,
    #[strum(to_string = "echo")]
    Echo,
    #[strum(to_string = "help")]
    Help,
    #[strum(to_string = "type")]
    Type,
    #[strum(to_string = "pwd")]
    Pwd,
    #[strum(to_string = "cd")]
    Cd,
    #[strum(to_string = "history")]
    History,
    #[strum(to_string = "jobs")]
    Jobs,
}
impl BuiltinCommands {
    pub fn is_builtin_command(s: &str) -> bool {
        s.parse::<BuiltinCommands>().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn recognizes_every_registered_builtin_name() {
        for cmd in BuiltinCommands::iter() {
            let name = cmd.to_string();
            assert_eq!(name.parse::<BuiltinCommands>().unwrap(), cmd);
            assert!(BuiltinCommands::is_builtin_command(&name));
        }
    }

    #[test]
    fn unknown_command_is_not_a_builtin() {
        assert!("not-a-real-command".parse::<BuiltinCommands>().is_err());
        assert!(!BuiltinCommands::is_builtin_command("ls"));
    }

    #[test]
    fn display_round_trips_through_the_lookup_table() {
        for cmd in BuiltinCommands::iter() {
            let name = cmd.to_string();
            assert_eq!(name.parse::<BuiltinCommands>().unwrap(), cmd);
        }
    }

    #[test]
    fn lookup_is_case_sensitive() {
        assert!("ECHO".parse::<BuiltinCommands>().is_err());
        assert!("Echo".parse::<BuiltinCommands>().is_err());
    }
}
