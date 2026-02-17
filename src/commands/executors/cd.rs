use std::{env, path::Path as StdPath};

pub fn execute_cd(args: &[String]) {
    if args.is_empty() {
        return;
    }
    let target = &args[0];
    let target_dir = StdPath::new(target);

    if let Err(e) = env::set_current_dir(target_dir) {
        eprintln!("cd: {}: {}", target, e);
    }
}
