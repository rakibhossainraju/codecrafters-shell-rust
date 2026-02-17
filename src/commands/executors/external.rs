use crate::commands::ExternalCommand;
use crate::utils;

pub fn execute_external_command(external_cmd: ExternalCommand) {
    // Try to resolve the command
    if let Some(external_cmd_path) = external_cmd.path {
        // Command found, execute it
        let mut cmd = std::process::Command::new(external_cmd_path);
        let args = external_cmd
            .args
            .as_ref()
            .map_or(vec![], |args| args.to_vec());
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
    } else {
        // Command not found
        utils::print_command_not_found(external_cmd.name.as_str());
    }
}
