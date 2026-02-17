pub fn execute_clear() {
    // Clear the terminal screen
    // This is a simple implementation that works on Unix-like systems and Windows
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .expect("Failed to clear the screen");
    } else {
        std::process::Command::new("clear")
            .status()
            .expect("Failed to clear the screen");
    }
}
