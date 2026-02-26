use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum Descriptor {
    Stdin,
    Stdout,
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

pub struct ResolvedRedirections {
    pub stdout: Option<File>,
    pub stderr: Option<File>,
    pub stdin: Option<File>,
}

impl ResolvedRedirections {
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
    pub fn from_resolved(resolved: ResolvedRedirections) -> Self {
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
