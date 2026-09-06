use crate::error::{Result, ShellError};
use crate::parser::lexer::Token;
use crate::utils::{Descriptor, Redirection, RedirectionType};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, Clone, Default)]
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirection>,
}

#[derive(Debug)]
pub enum ASTNode {
    Simple(ParsedCommand),
    Pipeline(Vec<ParsedCommand>),
    Background(Box<ASTNode>),
    // And(Box<ASTNode>, Box<ASTNode>),
    // Or(Box<ASTNode>, Box<ASTNode>),
}
pub type ASTNodes = Vec<ASTNode>;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn parser(tokens: Vec<Token>) -> Result<ASTNodes> {
        Parser::new(tokens).parse()
    }
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Result<ASTNodes> {
        let mut commands = Vec::new();
        // In the future, this will call `self.parse_and_or()`
        // For now, the highest level we have is a pipeline.
        while self.tokens.peek().is_some() {
            let ast = self.parse_pipeline()?;
            if let Some(Token::Background) = self.tokens.peek() {
                self.tokens.next(); // Consumes '&'
                if let Some(Token::Background) = self.tokens.peek() {
                    return Err(ShellError::ParserSyntaxError(Token::Background));
                }
                commands.push(ASTNode::Background(Box::new(ast)));
            } else {
                commands.push(ast);
            }
        }

        if commands.is_empty() {
            return Err(ShellError::SyntaxError(
                "unexpected empty command".to_string(),
            ));
        }

        Ok(commands)
    }

    fn parse_pipeline(&mut self) -> Result<ASTNode> {
        let mut commands = Vec::new();
        // 1. Parse the very first command
        commands.push(self.parse_simple_command()?);

        // 2. While the NEXT token is a pipe, consume it and parse another command!
        while let Some(Token::Pipe) = self.tokens.peek() {
            self.tokens.next(); // Consume the '|' token

            // Because we expect a command after a '|', this will naturally
            // throw our syntax error if it's empty (e.g., trailing pipe).
            commands.push(self.parse_simple_command()?);
        }

        if commands.len() == 1 {
            Ok(ASTNode::Simple(commands.pop().unwrap()))
        } else {
            Ok(ASTNode::Pipeline(commands))
        }
    }

    fn parse_simple_command(&mut self) -> Result<ParsedCommand> {
        let mut cmd = ParsedCommand::default();
        // Remembers the operator token that ended this command, if any, so
        // that an empty command can report *which* unexpected token caused
        // it (e.g. a leading `|` or `&`) rather than a generic message.
        let mut terminating_token = None;

        // Keep peeking at tokens until we hit an operator or run out of tokens
        while let Some(token) = self.tokens.peek() {
            match token {
                // If we see an operator, we STOP parsing this simple.
                // We leave the token in the iterator for `parse_pipeline` to find.
                Token::Pipe | Token::And | Token::Or | Token::Background => {
                    terminating_token = Some(token.clone());
                    break;
                }
                _ => {
                    let token = self.tokens.next().expect("guaranteed by peek");
                    match token {
                        Token::Word(word) => {
                            if cmd.cmd.is_empty() {
                                cmd.cmd = word;
                            } else {
                                cmd.args.push(word);
                            }
                        }
                        Token::RedirectOut(desc) => {
                            self.parse_redirect(&mut cmd, desc, RedirectionType::Output)?
                        }
                        Token::RedirectAppend(desc) => {
                            self.parse_redirect(&mut cmd, desc, RedirectionType::Append)?
                        }
                        Token::RedirectIn(desc) => {
                            self.parse_redirect(&mut cmd, desc, RedirectionType::Input)?
                        }
                        unexpected => return Err(ShellError::ParserSyntaxError(unexpected)),
                    }
                }
            }
        }
        if cmd.cmd.is_empty() {
            return Err(match terminating_token {
                Some(token) => ShellError::ParserSyntaxError(token),
                None => ShellError::SyntaxError("unexpected empty command".to_string()),
            });
        }
        Ok(cmd)
    }

    fn parse_redirect(
        &mut self,
        cmd: &mut ParsedCommand,
        descriptor: Descriptor,
        redirection_type: RedirectionType,
    ) -> Result<()> {
        match self.tokens.next() {
            Some(Token::Word(filename)) => {
                cmd.redirects.push(Redirection {
                    descriptor,
                    file: filename,
                    redirection_type,
                });
                Ok(())
            }
            _ => {
                let redir_symbol = match redirection_type {
                    RedirectionType::Input => "<",
                    RedirectionType::Output => ">",
                    RedirectionType::Append => ">>",
                };
                Err(ShellError::SyntaxError(format!(
                    "expected file name after {}",
                    redir_symbol
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Lexer;

    fn parse(input: &str) -> Result<ASTNodes> {
        let tokens = Lexer::tokenizer(input).expect("lexing should succeed");
        Parser::parser(tokens)
    }

    /// Parses input expected to produce exactly one top-level item and
    /// returns that item (unwrapped from the `ASTNodes` vec).
    fn parse_one(input: &str) -> ASTNode {
        let mut nodes = parse(input).unwrap();
        assert_eq!(
            nodes.len(),
            1,
            "expected exactly one top-level node, got {:?}",
            nodes
        );
        nodes.pop().unwrap()
    }

    #[test]
    fn parses_simple_command_with_args() {
        match parse_one("echo hello world") {
            ASTNode::Simple(cmd) => {
                assert_eq!(cmd.cmd, "echo");
                assert_eq!(cmd.args, vec!["hello".to_string(), "world".to_string()]);
                assert!(cmd.redirects.is_empty());
            }
            other => panic!("expected Simple, got {:?}", other),
        }
    }

    #[test]
    fn parses_pipeline_into_multiple_commands() {
        match parse_one("echo hi | tr a-z A-Z | wc -l") {
            ASTNode::Pipeline(cmds) => {
                assert_eq!(cmds.len(), 3);
                assert_eq!(cmds[0].cmd, "echo");
                assert_eq!(cmds[1].cmd, "tr");
                assert_eq!(cmds[1].args, vec!["a-z".to_string(), "A-Z".to_string()]);
                assert_eq!(cmds[2].cmd, "wc");
                assert_eq!(cmds[2].args, vec!["-l".to_string()]);
            }
            other => panic!("expected Pipeline, got {:?}", other),
        }
    }

    #[test]
    fn parses_output_redirection() {
        match parse_one("echo hi > out.txt") {
            ASTNode::Simple(cmd) => {
                assert_eq!(cmd.redirects.len(), 1);
                assert_eq!(cmd.redirects[0].file, "out.txt");
                assert_eq!(cmd.redirects[0].descriptor, Descriptor::Stdout);
                assert_eq!(cmd.redirects[0].redirection_type, RedirectionType::Output);
            }
            other => panic!("expected Simple, got {:?}", other),
        }
    }

    #[test]
    fn parses_append_and_input_and_stderr_redirection_together() {
        match parse_one("cmd 0< in.txt >> out.txt 2> err.txt") {
            ASTNode::Simple(cmd) => {
                assert_eq!(cmd.redirects.len(), 3);
                assert_eq!(cmd.redirects[0].redirection_type, RedirectionType::Input);
                assert_eq!(cmd.redirects[0].descriptor, Descriptor::Stdin);
                assert_eq!(cmd.redirects[1].redirection_type, RedirectionType::Append);
                assert_eq!(cmd.redirects[1].descriptor, Descriptor::Stdout);
                assert_eq!(cmd.redirects[2].redirection_type, RedirectionType::Output);
                assert_eq!(cmd.redirects[2].descriptor, Descriptor::Stderr);
            }
            other => panic!("expected Simple, got {:?}", other),
        }
    }

    #[test]
    fn empty_command_is_a_syntax_error() {
        assert!(parse("").is_err());
    }

    #[test]
    fn trailing_pipe_is_a_syntax_error() {
        let err = parse("echo hi |").unwrap_err();
        assert!(err.to_string().contains("unexpected empty command"));
    }

    #[test]
    fn leading_pipe_is_a_syntax_error() {
        assert!(parse("| echo hi").is_err());
    }

    #[test]
    fn redirect_missing_filename_is_a_syntax_error() {
        let err = parse("echo hi >").unwrap_err();
        assert!(err.to_string().contains("expected file name after >"));
    }

    /// Documents current (incomplete) behavior: `&&`/`||` are tokenized but
    /// the parser has no And/Or handling yet, so anything after the first
    /// pipeline segment is silently dropped rather than erroring or being
    /// executed. `&` (Background) is no longer part of this gap — see the
    /// `background_*` tests below.
    #[test]
    fn and_or_operators_still_silently_truncate_the_command() {
        match parse_one("echo a && echo b") {
            ASTNode::Simple(cmd) => {
                assert_eq!(cmd.cmd, "echo");
                assert_eq!(cmd.args, vec!["a".to_string()]);
            }
            other => panic!("expected Simple, got {:?}", other),
        }
    }

    #[test]
    fn single_trailing_background_wraps_command_in_background_node() {
        match parse_one("sleep 10 &") {
            ASTNode::Background(inner) => match *inner {
                ASTNode::Simple(cmd) => {
                    assert_eq!(cmd.cmd, "sleep");
                    assert_eq!(cmd.args, vec!["10".to_string()]);
                }
                other => panic!("expected Simple inside Background, got {:?}", other),
            },
            other => panic!("expected Background, got {:?}", other),
        }
    }

    #[test]
    fn background_wraps_the_whole_preceding_pipeline_not_just_its_last_stage() {
        match parse_one("echo hi | tr a-z A-Z &") {
            ASTNode::Background(inner) => match *inner {
                ASTNode::Pipeline(cmds) => {
                    assert_eq!(cmds.len(), 2);
                    assert_eq!(cmds[0].cmd, "echo");
                    assert_eq!(cmds[1].cmd, "tr");
                }
                other => panic!("expected Pipeline inside Background, got {:?}", other),
            },
            other => panic!("expected Background, got {:?}", other),
        }
    }

    #[test]
    fn only_the_command_before_the_ampersand_is_backgrounded() {
        let mut nodes = parse("sleep 2 & sleep 3").unwrap();
        assert_eq!(nodes.len(), 2);
        let second = nodes.pop().unwrap();
        let first = nodes.pop().unwrap();

        match first {
            ASTNode::Background(inner) => match *inner {
                ASTNode::Simple(cmd) => assert_eq!(cmd.args, vec!["2".to_string()]),
                other => panic!("expected Simple inside Background, got {:?}", other),
            },
            other => panic!("expected first item to be Background, got {:?}", other),
        }
        match second {
            ASTNode::Simple(cmd) => assert_eq!(cmd.args, vec!["3".to_string()]),
            other => panic!("expected second item to be plain Simple, got {:?}", other),
        }
    }

    #[test]
    fn trailing_ampersand_backgrounds_every_item_including_the_last() {
        let mut nodes = parse("sleep 2 & sleep 3 &").unwrap();
        assert_eq!(nodes.len(), 2);
        let second = nodes.pop().unwrap();
        let first = nodes.pop().unwrap();

        assert!(matches!(first, ASTNode::Background(_)));
        assert!(matches!(second, ASTNode::Background(_)));
    }

    #[test]
    fn leading_ampersand_is_a_syntax_error() {
        assert!(parse("& echo hi").is_err());
    }

    #[test]
    fn empty_item_between_two_ampersands_is_a_syntax_error() {
        let err = parse("sleep 2 & & sleep 3").unwrap_err();
        assert!(
            err.to_string()
                .contains("syntax error near unexpected token `&`")
        );
    }
}
