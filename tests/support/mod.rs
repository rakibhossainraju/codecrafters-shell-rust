//! Sandbox harness for black-box tests of the compiled shell binary.
//!
//! Every test spawns the real `codecrafters-shell` binary as a subprocess
//! with:
//!   - `env_clear()` — none of the real environment leaks in.
//!   - `PATH` pointing only at a throwaway temp "bin" directory containing a
//!     handful of tiny fixture scripts (see `install_fixture_bins`), so
//!     external-command resolution never touches the host's real PATH.
//!   - `HOME` / `HISTFILE` / cwd all pointing inside a `tempfile::TempDir`
//!     that is deleted when the `Sandbox` is dropped.
//!
//! Nothing a test does can read or write outside that temp directory.

#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

pub struct Sandbox {
    _root: TempDir,
    pub work_dir: PathBuf,
    pub home_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub histfile: PathBuf,
}

impl Sandbox {
    pub fn new() -> Self {
        let root = TempDir::new().expect("create sandbox tempdir");
        let work_dir = root.path().join("work");
        let home_dir = root.path().join("home");
        let bin_dir = root.path().join("bin");
        let histfile = root.path().join("histfile");
        fs::create_dir_all(&work_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();

        let sandbox = Self {
            _root: root,
            work_dir,
            home_dir,
            bin_dir,
            histfile,
        };
        sandbox.install_fixture_bins();
        sandbox
    }

    fn install_fixture_bins(&self) {
        // Prints each argument on its own line (lets tests assert exact
        // word-splitting/quoting behavior, unlike joining with spaces).
        self.install_fake_bin(
            "argecho",
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done\n",
        );
        // Cats stdin to stdout, using the real /bin/cat by absolute path
        // (absolute paths bypass PATH lookup, so this doesn't depend on the
        // sandboxed PATH at all).
        self.install_fake_bin("catit", "#!/bin/sh\nexec /bin/cat\n");
        // Uppercases stdin.
        self.install_fake_bin("upper", "#!/bin/sh\nexec /usr/bin/tr 'a-z' 'A-Z'\n");
        // Writes a fixed line to stderr only.
        self.install_fake_bin("stderrer", "#!/bin/sh\necho 'err message' 1>&2\n");
        // Exits with a non-zero status and no output.
        self.install_fake_bin("failer", "#!/bin/sh\nexit 3\n");
        // Fake `clear` so the `clear` builtin (which shells out to a real
        // `clear` binary) has something deterministic to find on PATH.
        self.install_fake_bin("clear", "#!/bin/sh\nprintf 'CLEARED\\n'\n");
    }

    fn install_fake_bin(&self, name: &str, contents: &str) {
        let path = self.bin_dir.join(name);
        fs::write(&path, contents).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    pub fn shell_bin() -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_codecrafters-shell"))
    }

    /// Feed `script` (one shell command per line) to a freshly spawned,
    /// fully sandboxed shell process and collect its output.
    pub fn run(&self, script: &str) -> Output {
        let mut child = Command::new(Self::shell_bin())
            .current_dir(&self.work_dir)
            .env_clear()
            .env("PATH", &self.bin_dir)
            .env("HOME", &self.home_dir)
            .env("HISTFILE", &self.histfile)
            .env("TERM", "dumb")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn sandboxed shell");

        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .expect("write script to sandboxed shell stdin");

        child
            .wait_with_output()
            .expect("wait for sandboxed shell to exit")
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.work_dir.join(relative)
    }

    pub fn write_file(&self, relative: &str, contents: &str) {
        let full = self.path(relative);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, contents).unwrap();
    }

    pub fn read_file(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap()
    }

    pub fn file_exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }
}

/// Captured stdout with the `"$ "` prompt stripped out.
///
/// The shell's `readline("$ ")` writes that prompt straight to stdout on
/// every call, even when stdin/stdout are piped rather than a real tty, so
/// raw captured output looks like `"$ $ hello\n$ "` for a one-line script.
/// None of this test suite's fixture output ever contains the literal
/// substring `"$ "`, so a global strip cleanly recovers just the command
/// output.
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("$ ", "")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
