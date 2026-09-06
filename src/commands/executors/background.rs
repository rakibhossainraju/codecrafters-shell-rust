use crate::{
    commands::{BuiltinCommands, Command, ast_executor, executors::pipeline::execute_pipeline},
    error::{Result, ShellError},
    parser::{ASTNode, ParsedCommand},
    state::{Job, ShellState},
};
use std::io::{Read, Write};

pub fn execute_background(ast: Box<ASTNode>, state: &mut ShellState) -> Result<()> {
    match *ast {
        ASTNode::Simple(parsed_cmd) => {
            let cmd = Command::resolve(parsed_cmd)?;

            match cmd {
                Command::Builtin(BuiltinCommands::Exit, _) => {
                    return Err(ShellError::ExitOut);
                }
                Command::External(external_cmd) => {
                    let child = external_cmd.spawn(None, None)?;
                    state.jobs.push_job(child);
                }
                builtin_cmd => {
                    builtin_cmd.execute(None, None, state)?;
                }
            }
        }
        ASTNode::Pipeline(cmds) => {
            execute_pipeline(cmds, state)?;
        }
        ASTNode::Background(_) => {
            unreachable!("the parser never nests Background inside Background")
        }
    }
    // println!("CHILDREN_WAITING: {:#?}", children);
    Ok(())
}
