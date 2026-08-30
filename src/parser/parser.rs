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
    // Background(Box<ASTNode>, Box<ASTNode>),
    // And(Box<ASTNode>, Box<ASTNode>),
    // Or(Box<ASTNode>, Box<ASTNode>),
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn parser(tokens: Vec<Token>) -> Result<ASTNode> {
        Parser::new(tokens).parse()
    }
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Result<ASTNode> {
        // In the future, this will call `self.parse_and_or()`
        // For now, the highest level we have is a pipeline.
        self.parse_pipeline()
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

        // Keep peeking at tokens until we hit an operator or run out of tokens
        while let Some(token) = self.tokens.peek() {
            match token {
                // If we see an operator, we STOP parsing this simple.
                // We leave the token in the iterator for `parse_pipeline` to find.
                Token::Pipe | Token::And | Token::Or | Token::Background => break,
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
                        unknown_token => {
                            return Err(ShellError::SyntaxError(format!(
                                "unexpected token in simple command: {:?}",
                                unknown_token
                            )));
                        }
                    }
                }
            }
        }
        if cmd.cmd.is_empty() {
            return Err(ShellError::SyntaxError(
                "unexpected empty command".to_string(),
            ));
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

    // fn parse_background(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Lexer;

    fn parse(input: &str) -> Result<ASTNode> {
        let tokens = Lexer::tokenizer(input).expect("lexing should succeed");
        Parser::parser(tokens)
    }

    #[test]
    fn parses_simple_command_with_args() {
        let ast = parse("echo hello world").unwrap();
        match ast {
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
        let ast = parse("echo hi | tr a-z A-Z | wc -l").unwrap();
        match ast {
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
        let ast = parse("echo hi > out.txt").unwrap();
        match ast {
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
        let ast = parse("cmd 0< in.txt >> out.txt 2> err.txt").unwrap();
        match ast {
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

    /// Documents current (incomplete) behavior: `&&`/`||`/`&` are tokenized
    /// but the parser has no And/Or/Background handling yet, so anything
    /// after the first pipeline segment is silently dropped rather than
    /// erroring or being executed.
    #[test]
    fn and_or_background_operators_silently_truncate_the_command() {
        let ast = parse("echo a && echo b").unwrap();
        match ast {
            ASTNode::Simple(cmd) => {
                assert_eq!(cmd.cmd, "echo");
                assert_eq!(cmd.args, vec!["a".to_string()]);
            }
            other => panic!("expected Simple, got {:?}", other),
        }
    }
}
