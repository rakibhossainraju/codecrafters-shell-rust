use crate::commands::ExternalCommand;
use crate::utils::print_command_not_found;

pub fn execute_external_command(ext: &ExternalCommand) {
    // TODO: Implement actual execution logic here (e.g., using std::process::Command)
    // if ext.path.is_none() {
    //     // Try to resolve the command
    //     if let Some(resolved) = ExternalCommand::resolve(&ext.name) {
    //         // Command exists but we don't execute it yet (future stage)
    //     } else {
    //         // Command not found
    //         eprintln!("{}: command not found", ext.name);
    //     }
    // } else {
    // }

    // For now, we just print that the command was not found since execution is not implemented yet
    print_command_not_found(ext.name.as_str());
}