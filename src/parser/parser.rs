use crate::error::{Result, ShellError};
use crate::parser::lexer::Token;
use crate::utils::{Descriptor, Redirection, RedirectionType};
use std::iter::Peekable;
use std::mem;
use std::vec::IntoIter;

#[derive(Debug, Default)]
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirection>,
}

pub struct Parser {
    parsed_command: ParsedCommand,
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn parser(tokens: Vec<Token>) -> Result<ParsedCommand> {
        Parser::new(tokens).parse()
    }
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            parsed_command: ParsedCommand::default(),
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Result<ParsedCommand> {
        while let Some(token) = self.tokens.next() {
            match token {
                Token::Word(word) => self.parse_word(word),
                Token::RedirectOut(desc) => self.parse_redirect(desc, RedirectionType::Output)?,
                Token::RedirectAppend(desc) => {
                    self.parse_redirect(desc, RedirectionType::Append)?
                }
                Token::RedirectIn(desc) => self.parse_redirect(desc, RedirectionType::Input)?,
                _ => {
                    // NOT IMPLEMENTED YET.
                    break;
                }
            }
        }
        Ok(mem::take(&mut self.parsed_command))
    }

    fn parse_word(&mut self, word: String) {
        if self.parsed_command.cmd.is_empty() {
            self.parsed_command.cmd = word;
        } else {
            self.parsed_command.args.push(word);
        }
    }

    fn parse_redirect(
        &mut self,
        descriptor: Descriptor,
        redirection_type: RedirectionType,
    ) -> Result<()> {
        // We hit a `>`, `>>`, or `<`. The VERY NEXT token MUST be a Word (the filename).
        // We consume it immediately using `next()`.
        match self.tokens.next() {
            Some(Token::Word(filename)) => {
                self.parsed_command.redirects.push(Redirection {
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
