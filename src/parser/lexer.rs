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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::tokenizer(input).expect("expected successful tokenization")
    }

    #[test]
    fn tokenizes_plain_words() {
        assert_eq!(
            tokenize("echo hello world"),
            vec![
                Token::Word("echo".into()),
                Token::Word("hello".into()),
                Token::Word("world".into()),
            ]
        );
    }

    #[test]
    fn collapses_extra_whitespace() {
        assert_eq!(
            tokenize("  echo    hi  "),
            vec![Token::Word("echo".into()), Token::Word("hi".into())]
        );
    }

    #[test]
    fn single_quotes_are_fully_literal() {
        // Nothing is special inside single quotes, not even backslash.
        assert_eq!(
            tokenize(r"echo 'a\ b  c'"),
            vec![Token::Word("echo".into()), Token::Word(r"a\ b  c".into())]
        );
    }

    #[test]
    fn double_quotes_preserve_spacing_but_allow_escaping_quote_and_backslash() {
        assert_eq!(
            tokenize(r#"echo "say \"hi\" and \\ done""#),
            vec![
                Token::Word("echo".into()),
                Token::Word(r#"say "hi" and \ done"#.into()),
            ]
        );
    }

    #[test]
    fn double_quotes_keep_unrecognized_escapes_literal() {
        // Rule: inside double quotes only \" and \\ are actual escapes.
        assert_eq!(
            tokenize(r#"echo "a\nb""#),
            vec![Token::Word("echo".into()), Token::Word(r"a\nb".into())]
        );
    }

    #[test]
    fn unquoted_backslash_escapes_next_char_and_is_consumed() {
        assert_eq!(
            tokenize(r"echo hello\ world"),
            vec![Token::Word("echo".into()), Token::Word("hello world".into())]
        );
    }

    #[test]
    fn adjacent_quoted_and_unquoted_segments_merge_into_one_word() {
        assert_eq!(
            tokenize(r#"echo foo"bar"'baz'"#),
            vec![Token::Word("echo".into()), Token::Word("foobarbaz".into())]
        );
    }

    #[test]
    fn recognizes_pipe_and_or_tokens() {
        assert_eq!(
            tokenize("a | b || c"),
            vec![
                Token::Word("a".into()),
                Token::Pipe,
                Token::Word("b".into()),
                Token::Or,
                Token::Word("c".into()),
            ]
        );
    }

    #[test]
    fn recognizes_background_and_and_tokens() {
        assert_eq!(
            tokenize("a & b && c"),
            vec![
                Token::Word("a".into()),
                Token::Background,
                Token::Word("b".into()),
                Token::And,
                Token::Word("c".into()),
            ]
        );
    }

    #[test]
    fn plain_redirect_out_and_append() {
        assert_eq!(
            tokenize("echo hi > out.txt"),
            vec![
                Token::Word("echo".into()),
                Token::Word("hi".into()),
                Token::RedirectOut(Descriptor::Stdout),
                Token::Word("out.txt".into()),
            ]
        );
        assert_eq!(
            tokenize("echo hi >> out.txt"),
            vec![
                Token::Word("echo".into()),
                Token::Word("hi".into()),
                Token::RedirectAppend(Descriptor::Stdout),
                Token::Word("out.txt".into()),
            ]
        );
    }

    #[test]
    fn numeric_descriptor_redirects() {
        assert_eq!(
            tokenize("cmd 2> err.txt"),
            vec![
                Token::Word("cmd".into()),
                Token::RedirectOut(Descriptor::Stderr),
                Token::Word("err.txt".into()),
            ]
        );
        assert_eq!(
            tokenize("cmd 1>> out.txt"),
            vec![
                Token::Word("cmd".into()),
                Token::RedirectAppend(Descriptor::Stdout),
                Token::Word("out.txt".into()),
            ]
        );
        assert_eq!(
            tokenize("cmd 0< in.txt"),
            vec![
                Token::Word("cmd".into()),
                Token::RedirectIn(Descriptor::Stdin),
                Token::Word("in.txt".into()),
            ]
        );
    }

    #[test]
    fn digit_not_followed_by_redirect_is_a_plain_word_char() {
        assert_eq!(tokenize("echo 2plus2"), vec![
            Token::Word("echo".into()),
            Token::Word("2plus2".into()),
        ]);
    }

    #[test]
    fn unclosed_single_quote_is_a_syntax_error() {
        let err = Lexer::tokenizer("echo 'unterminated").unwrap_err();
        assert!(err.to_string().contains("unclosed single quote"));
    }

    #[test]
    fn unclosed_double_quote_is_a_syntax_error() {
        let err = Lexer::tokenizer(r#"echo "unterminated"#).unwrap_err();
        assert!(err.to_string().contains("unclosed double quote"));
    }

    #[test]
    fn trailing_backslash_is_a_syntax_error() {
        let err = Lexer::tokenizer(r"echo trailing\").unwrap_err();
        assert!(err.to_string().contains("unclosed escape sequence"));
    }
}
