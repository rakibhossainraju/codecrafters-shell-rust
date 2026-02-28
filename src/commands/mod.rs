mod builtin;
mod command;
mod executors;
mod external;

use crate::commands::executors::pipeline::execute_pipeline;
use crate::error::{Result, ShellError};
use crate::parser::ASTNode;
use crate::state::ShellState;
pub use builtin::*;
pub use command::*;
pub use external::*;

pub fn execute_ast(ast: ASTNode, state: &mut ShellState) -> Result<()> {
    match ast {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = Command::resolve(parsed_cmd)?;

            // Check for exit before executing (to break the loop)
            if matches!(cmd, Command::Builtin(BuiltinCommands::Exit, _)) {
                return Err(ShellError::ExitOut);
            }

            // Execute the command with remaining arguments
            cmd.execute(None, None, state)?;
            // If we get here, the command executed successfully
            Ok(())
        }
        ASTNode::Pipeline(cmds) => {
            execute_pipeline(cmds, state)?;
            Ok(())
        }
    }
}
