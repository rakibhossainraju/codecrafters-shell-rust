use crate::error::{Result, ShellError};
use crate::parser::lexer::Token;
use std::iter::Peekable;
use std::mem;
use std::vec::IntoIter;

#[derive(Debug, Default)]
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub redirect_in: Option<String>,
    pub redirect_out: Option<String>,
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
                Token::RedirectOut => self.parse_redirect_out()?,
                Token::RedirectIn => self.parse_redirect_in()?,
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

    fn parse_redirect_out(&mut self) -> Result<()> {
        // We hit a `>`. The VERY NEXT token MUST be a Word (the filename).
        // We consume it immediately using `next()`.
        match self.tokens.next() {
            Some(Token::Word(filename)) => {
                self.parsed_command.redirect_out = Some(filename);
                Ok(())
            }
            _ => Err(ShellError::SyntaxError(
                "expected file name after >".to_string(),
            )),
        }
    }

    fn parse_redirect_in(&mut self) -> Result<()> {
        match self.tokens.next() {
            Some(Token::Word(filename)) => {
                self.parsed_command.redirect_in = Some(filename);
                Ok(())
            }
            _ => Err(ShellError::SyntaxError(
                "expected file name after <".to_string(),
            )),
        }
    }

    // fn parse_background(&mut self) {}
}
