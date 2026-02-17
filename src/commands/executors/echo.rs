/// Execute the echo builtin command
pub fn execute_echo(args: &[String]) {
    if args.is_empty() {
        println!();
    } else {
        println!("{}", args.join(" "));
    }
}
