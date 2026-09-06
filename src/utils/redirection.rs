use strum::{Display, EnumString};

use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Descriptor {
    #[strum(to_string = "")]
    Stdin,
    #[strum(to_string = "")]
    Stdout,
    #[strum(to_string = "2")]
    Stderr,
}

impl From<char> for Descriptor {
    fn from(s: char) -> Self {
        match s {
            '0' => Descriptor::Stdin,
            '1' => Descriptor::Stdout,
            '2' => Descriptor::Stderr,
            _ => panic!("Invalid descriptor: {}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionType {
    Input,
    Output,
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    pub descriptor: Descriptor,
    pub file: String,
    pub redirection_type: RedirectionType,
}

pub struct ResolvedReDirections {
    pub stdout: Option<File>,
    pub stderr: Option<File>,
    pub stdin: Option<File>,
}

impl ResolvedReDirections {
    pub fn resolve(parsed_cmd: &ParsedCommand) -> Result<Self> {
        let mut stdout = None;
        let mut stderr = None;
        let mut stdin = None;

        for redirect in &parsed_cmd.redirects {
            let file = match redirect.redirection_type {
                RedirectionType::Output => OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&redirect.file),
                RedirectionType::Append => OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&redirect.file),
                RedirectionType::Input => OpenOptions::new().read(true).open(&redirect.file),
            }
            .map_err(|e| {
                eprintln!("shell: {}: {}", redirect.file, e);
                ShellError::IoError(e)
            })?;

            match redirect.descriptor {
                Descriptor::Stdout => stdout = Some(file),
                Descriptor::Stderr => stderr = Some(file),
                Descriptor::Stdin => stdin = Some(file),
            }
        }

        Ok(Self {
            stdout,
            stderr,
            stdin,
        })
    }
}

pub struct IoStreams {
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
    pub stdin: Box<dyn Read>,
}

impl IoStreams {
    pub fn from_resolved(resolved: ResolvedReDirections) -> Self {
        let stdout: Box<dyn Write> = match resolved.stdout {
            Some(f) => Box::new(f),
            None => Box::new(io::stdout()),
        };
        let stderr: Box<dyn Write> = match resolved.stderr {
            Some(f) => Box::new(f),
            None => Box::new(io::stderr()),
        };
        let stdin: Box<dyn Read> = match resolved.stdin {
            Some(f) => Box::new(f),
            None => Box::new(io::stdin()),
        };
        Self {
            stdout,
            stderr,
            stdin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;
    use tempfile::TempDir;

    fn cmd_with(redirects: Vec<Redirection>) -> ParsedCommand {
        ParsedCommand {
            cmd: "cmd".to_string(),
            args: vec![],
            redirects,
        }
    }

    #[test]
    fn output_redirect_truncates_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.txt");
        std::fs::write(&path, "old content that should disappear").unwrap();

        let resolved = ResolvedReDirections::resolve(&cmd_with(vec![Redirection {
            descriptor: Descriptor::Stdout,
            file: path.to_str().unwrap().to_string(),
            redirection_type: RedirectionType::Output,
        }]))
        .unwrap();

        let mut file = resolved.stdout.expect("stdout file should be opened");
        write!(file, "new").unwrap();
        drop(file);

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    fn append_redirect_keeps_existing_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.txt");
        std::fs::write(&path, "first-").unwrap();

        let resolved = ResolvedReDirections::resolve(&cmd_with(vec![Redirection {
            descriptor: Descriptor::Stdout,
            file: path.to_str().unwrap().to_string(),
            redirection_type: RedirectionType::Append,
        }]))
        .unwrap();

        let mut file = resolved.stdout.expect("stdout file should be opened");
        write!(file, "second").unwrap();
        drop(file);

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first-second");
    }

    #[test]
    fn input_redirect_opens_the_file_read_only() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("in.txt");
        std::fs::write(&path, "hello from file").unwrap();

        let resolved = ResolvedReDirections::resolve(&cmd_with(vec![Redirection {
            descriptor: Descriptor::Stdin,
            file: path.to_str().unwrap().to_string(),
            redirection_type: RedirectionType::Input,
        }]))
        .unwrap();

        let mut file = resolved.stdin.expect("stdin file should be opened");
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "hello from file");
    }

    #[test]
    fn missing_input_file_is_an_io_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.txt");

        let result = ResolvedReDirections::resolve(&cmd_with(vec![Redirection {
            descriptor: Descriptor::Stdin,
            file: path.to_str().unwrap().to_string(),
            redirection_type: RedirectionType::Input,
        }]));

        assert!(matches!(result, Err(ShellError::IoError(_))));
    }

    #[test]
    fn multiple_redirects_resolve_independently() {
        let dir = TempDir::new().unwrap();
        let out_path = dir.path().join("out.txt");
        let err_path = dir.path().join("err.txt");

        let resolved = ResolvedReDirections::resolve(&cmd_with(vec![
            Redirection {
                descriptor: Descriptor::Stdout,
                file: out_path.to_str().unwrap().to_string(),
                redirection_type: RedirectionType::Output,
            },
            Redirection {
                descriptor: Descriptor::Stderr,
                file: err_path.to_str().unwrap().to_string(),
                redirection_type: RedirectionType::Output,
            },
        ]))
        .unwrap();

        assert!(resolved.stdout.is_some());
        assert!(resolved.stderr.is_some());
        assert!(resolved.stdin.is_none());
    }

    #[test]
    fn no_redirects_resolves_to_all_none() {
        let resolved = ResolvedReDirections::resolve(&cmd_with(vec![])).unwrap();
        assert!(resolved.stdout.is_none());
        assert!(resolved.stderr.is_none());
        assert!(resolved.stdin.is_none());
    }
}
