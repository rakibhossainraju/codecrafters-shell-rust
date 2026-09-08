//! Serializes/deserializes an entire pipeline (`Vec<ParsedCommand>`,
//! including each stage's own redirects) into a single string that can be
//! passed as one argv entry to a re-exec'd child process.
//!
//! Used by `executors::background::spawn_pipeline_job` (encode, in the
//! parent, before spawning) and `commands::run_internal_pipeline` (decode,
//! in the re-exec'd child). See `executors::background` for why
//! backgrounding a pipeline re-execs the shell binary at all.
//!
//! Every string is written length-prefixed (`<byte-len>:<bytes>`) rather
//! than joined with a delimiter like `|`. A delimiter can always collide
//! with a real argument that happens to contain that exact character
//! (quoted or not); an explicit byte count can't collide with anything,
//! since decoding never needs to search for a boundary -- it just reads
//! exactly that many bytes.

use crate::error::{Result, ShellError};
use crate::parser::ParsedCommand;
use crate::utils::redirection::{Descriptor, Redirection, RedirectionType};

pub fn encode_pipeline(cmds: &[ParsedCommand]) -> String {
    let mut out = String::new();
    write_usize(&mut out, cmds.len());
    for cmd in cmds {
        write_str(&mut out, &cmd.cmd);
        write_usize(&mut out, cmd.args.len());
        for arg in &cmd.args {
            write_str(&mut out, arg);
        }
        write_usize(&mut out, cmd.redirects.len());
        for r in &cmd.redirects {
            write_str(&mut out, descriptor_code(&r.descriptor));
            write_str(&mut out, &r.redirection_type.to_string());
            write_str(&mut out, &r.file);
        }
    }
    out
}

pub fn decode_pipeline(data: &str) -> Result<Vec<ParsedCommand>> {
    let mut pos = 0;
    let cmd_count = read_usize(data, &mut pos).ok_or_else(malformed)?;

    let mut cmds = Vec::with_capacity(cmd_count);
    for _ in 0..cmd_count {
        let cmd = read_str(data, &mut pos).ok_or_else(malformed)?;

        let arg_count = read_usize(data, &mut pos).ok_or_else(malformed)?;
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(read_str(data, &mut pos).ok_or_else(malformed)?);
        }

        let redirect_count = read_usize(data, &mut pos).ok_or_else(malformed)?;
        let mut redirects = Vec::with_capacity(redirect_count);
        for _ in 0..redirect_count {
            let descriptor = read_str(data, &mut pos).ok_or_else(malformed)?;
            let redirection_type = read_str(data, &mut pos).ok_or_else(malformed)?;
            let file = read_str(data, &mut pos).ok_or_else(malformed)?;
            redirects.push(Redirection {
                descriptor: descriptor_from_code(&descriptor).ok_or_else(malformed)?,
                redirection_type: redirection_type.parse().map_err(|_| malformed())?,
                file,
            });
        }

        cmds.push(ParsedCommand {
            cmd,
            args,
            redirects,
        });
    }
    Ok(cmds)
}

fn malformed() -> ShellError {
    ShellError::SyntaxError("malformed internal pipeline payload".to_string())
}

fn write_str(out: &mut String, s: &str) {
    out.push_str(&s.len().to_string());
    out.push(':');
    out.push_str(s);
}

fn write_usize(out: &mut String, n: usize) {
    out.push_str(&n.to_string());
    out.push(';');
}

fn read_str(data: &str, pos: &mut usize) -> Option<String> {
    let rest = data.get(*pos..)?;
    let colon = rest.find(':')?;
    let len: usize = rest[..colon].parse().ok()?;
    let start = *pos + colon + 1;
    let end = start + len;
    let s = data.get(start..end)?.to_string();
    *pos = end;
    Some(s)
}

fn read_usize(data: &str, pos: &mut usize) -> Option<usize> {
    let rest = data.get(*pos..)?;
    let semi = rest.find(';')?;
    let n: usize = rest[..semi].parse().ok()?;
    *pos += semi + 1;
    Some(n)
}

/// `Descriptor`'s own `Display` can't be reused here: it renders `Stdin`
/// and `Stdout` both as `""` (fine for pretty-printing `<file`/`>file`
/// without a leading descriptor, useless for round-tripping since they'd be
/// indistinguishable on decode). And `Descriptor`'s existing `From<char>`
/// panics on invalid input rather than returning `None`, which decoding a
/// payload defensively needs — so both directions stay hand-rolled here,
/// deliberately using the same '0'/'1'/'2' mapping as that `From<char>` impl.
fn descriptor_code(d: &Descriptor) -> &'static str {
    match d {
        Descriptor::Stdin => "0",
        Descriptor::Stdout => "1",
        Descriptor::Stderr => "2",
    }
}

fn descriptor_from_code(s: &str) -> Option<Descriptor> {
    match s {
        "0" => Some(Descriptor::Stdin),
        "1" => Some(Descriptor::Stdout),
        "2" => Some(Descriptor::Stderr),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_multi_stage_pipeline_with_redirects() {
        let cmds = vec![
            ParsedCommand {
                cmd: "echo".to_string(),
                args: vec!["hi there".to_string(), "a:b;c".to_string()],
                redirects: vec![Redirection {
                    descriptor: Descriptor::Stdin,
                    redirection_type: RedirectionType::Input,
                    file: "in.txt".to_string(),
                }],
            },
            ParsedCommand {
                cmd: "grep".to_string(),
                args: vec!["|weird|arg|".to_string()],
                redirects: vec![Redirection {
                    descriptor: Descriptor::Stdout,
                    redirection_type: RedirectionType::Append,
                    file: "out.txt".to_string(),
                }],
            },
        ];

        let encoded = encode_pipeline(&cmds);
        let decoded = decode_pipeline(&encoded).expect("should decode cleanly");

        // ParsedCommand doesn't derive PartialEq, so compare field-by-field.
        assert_eq!(decoded.len(), cmds.len());
        for (d, c) in decoded.iter().zip(cmds.iter()) {
            assert_eq!(d.cmd, c.cmd);
            assert_eq!(d.args, c.args);
            assert_eq!(d.redirects, c.redirects);
        }
    }

    #[test]
    fn round_trips_an_empty_pipeline() {
        let encoded = encode_pipeline(&[]);
        let decoded = decode_pipeline(&encoded).expect("should decode cleanly");
        assert!(decoded.is_empty());
    }

    #[test]
    fn rejects_garbage_payloads() {
        assert!(decode_pipeline("not a valid payload").is_err());
    }
}
