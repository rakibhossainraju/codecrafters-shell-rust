use crate::error::{Result, ShellError};
use crate::utils::Descriptor;
use std::iter::Peekable;
use std::mem;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,                       // |
    Or,                         // ||
    Background,                 // &
    And,                        // &&
    RedirectOut(Descriptor),    // >
    RedirectAppend(Descriptor), // >>
    RedirectIn(Descriptor),     // <
}

#[derive(Debug, PartialEq)]
enum LexerState {
    Normal,
    SingleQuote,
    DoubleQuote,
    Escape(Box<LexerState>), // optional for remembering previous state
}
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    current_arg: String,
    state: LexerState,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn tokenizer(input: &'a str) -> Result<Vec<Token>> {
        let mut laxer = Lexer::new(input);
        laxer.tokenize()
    }
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            current_arg: String::new(),
            state: LexerState::Normal,
            tokens: Vec::new(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        while let Some(c) = self.chars.next() {
            match self.state {
                LexerState::Normal => self.lex_normal(c),
                LexerState::SingleQuote => self.lex_single_quote(c),
                LexerState::DoubleQuote => self.lex_double_quote(c),
                LexerState::Escape(_) => self.lex_escapee(c),
            }
        }
        // Flush whatever is left in the buffer when the string ends!
        self.flush_current_word();

        match self.state {
            LexerState::Normal => Ok(mem::take(&mut self.tokens)),
            LexerState::SingleQuote => {
                Err(ShellError::SyntaxError("unclosed single quote".to_string()))
            }
            LexerState::DoubleQuote => {
                Err(ShellError::SyntaxError("unclosed double quote".to_string()))
            }
            LexerState::Escape(_) => Err(ShellError::SyntaxError(
                "unclosed escape sequence".to_string(),
            )),
        }
    }

    fn lex_normal(&mut self, c: char) {
        match c {
            '\'' => self.state = LexerState::SingleQuote,
            '"' => self.state = LexerState::DoubleQuote,
            '\\' => self.state = LexerState::Escape(Box::new(LexerState::Normal)),
            '0' | '1' | '2' => self.handle_descriptor(c),
            '>' => {
                self.flush_current_word();
                if self.chars.peek() == Some(&'>') {
                    self.chars.next();
                    self.tokens.push(Token::RedirectAppend(Descriptor::Stdout));
                } else {
                    self.tokens.push(Token::RedirectOut(Descriptor::Stdout));
                }
            }
            '<' => self.flush_current_word_then(Token::RedirectIn(Descriptor::Stdout)),
            '|' => {
                self.flush_current_word();
                if self.chars.peek() == Some(&'|') {
                    self.chars.next();
                    self.tokens.push(Token::Or);
                } else {
                    self.tokens.push(Token::Pipe);
                }
            }
            '&' => {
                self.flush_current_word();
                if self.chars.peek() == Some(&'&') {
                    self.chars.next();
                    self.tokens.push(Token::And);
                } else {
                    self.tokens.push(Token::Background);
                }
            }
            _ if c.is_whitespace() => self.flush_current_word(),
            _ => self.current_arg.push(c),
        }
    }

    fn lex_single_quote(&mut self, c: char) {
        match c {
            '\'' => self.state = LexerState::Normal,
            _ => self.current_arg.push(c),
        }
    }

    fn lex_double_quote(&mut self, c: char) {
        match c {
            '"' => self.state = LexerState::Normal,
            '\\' => self.state = LexerState::Escape(Box::new(LexerState::DoubleQuote)),
            _ => self.current_arg.push(c),
        }
    }

    fn lex_escapee(&mut self, c: char) {
        if let LexerState::Escape(state) = mem::replace(&mut self.state, LexerState::Normal) {
            match *state {
                // Rule: Inside double quotes, only \ and " are actually escaped
                LexerState::DoubleQuote => {
                    if c == '"' || c == '\\' {
                        self.current_arg.push(c);
                    } else {
                        self.current_arg.push('\\');
                        self.current_arg.push(c);
                    }
                }
                // Rule: Everywhere else, the backslash is consumed entirely
                _ => self.current_arg.push(c),
            }
            // Return to the previous state
            self.state = *state;
        } else {
            unreachable!("Escape state logic is broken");
        }
    }
}

impl<'a> Lexer<'a> {
    fn flush_current_word(&mut self) {
        if !self.current_arg.is_empty() {
            self.tokens
                .push(Token::Word(mem::take(&mut self.current_arg)));
        }
    }

    fn flush_current_word_then(&mut self, token: Token) {
        self.flush_current_word();
        self.tokens.push(token);
    }

    fn handle_descriptor(&mut self, c: char) {
        if self.chars.peek() == Some(&'>') {
            self.chars.next();
            if self.chars.peek() == Some(&'>') {
                self.chars.next();
                self.flush_current_word_then(Token::RedirectAppend(c.into()));
            } else {
                self.flush_current_word_then(Token::RedirectOut(c.into()));
            }
        } else if self.chars.peek() == Some(&'<') {
            self.chars.next();
            self.flush_current_word_then(Token::RedirectIn(c.into()));
        } else {
            self.current_arg.push(c);
        }
    }
}
