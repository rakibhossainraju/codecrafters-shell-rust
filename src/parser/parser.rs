use crate::error::{Result, ShellError};
use crate::parser::lexer::Token;
use crate::utils::{Descriptor, Redirection, RedirectionType};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, Default)]
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
