use crate::commands::ExternalCommand;
use crate::utils;
use std::os::unix::prelude::CommandExt;
use std::process::Command as OsCommand;

pub fn execute_external_command(external_cmd: &ExternalCommand) {
    // The path is already resolved in ExternalCommand
    let mut cmd = OsCommand::new(&external_cmd.path);

    // Set arg0 to the command name (not full path)
    cmd.arg0(&external_cmd.ast.cmd);

    // Add all arguments
    cmd.args(&external_cmd.ast.args);

    // TODO:: Add the redirect logic!
    // if let Some(out_file) = external_cmd.ast.redirect_out { ... }

    match cmd.spawn() {
        Ok(mut child) => {
            if let Ok(status) = child.wait() {
                if !status.success() {
                    utils::print_exit_with_status(&external_cmd.ast.cmd, status);
                }
            } else {
                eprintln!("Failed to wait for command '{}'", external_cmd.ast.cmd);
            }
        }
        Err(error) => {
            utils::print_failed_to_execute(&external_cmd.ast.cmd, error);
        }
    }
}
