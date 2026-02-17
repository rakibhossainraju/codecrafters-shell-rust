use crate::commands::ExternalCommand;
use crate::utils;
use std::os::unix::prelude::CommandExt;

pub fn execute_external_command(external_cmd: ExternalCommand) {
    let external_cmd_path = match external_cmd.path {
        Some(path) => path,
        None => {
            utils::print_command_not_found(external_cmd.name.as_str());
            return;
        }
    };
    let mut cmd = std::process::Command::new(external_cmd_path);
    let args = external_cmd.args.unwrap_or_default();

    cmd.arg0(external_cmd.name.as_str());
    cmd.args(&args);

    match cmd.spawn() {
        Ok(mut child) => {
            // Wait for the command to finish
            if let Ok(status) = child.wait() {
                if !status.success() {
                    utils::print_exit_with_status(external_cmd.name.as_str(), status);
                    return;
                }
            } else {
                eprintln!("Failed to wait for command '{}'", external_cmd.name);
            }
        }
        Err(e) => {
            utils::print_filed_to_execute(external_cmd.name.as_str(), e);
        }
    }
}
