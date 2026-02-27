use std::{ops::Range, rc::Rc};

use ariadne::{ColorGenerator, Label, Report, ReportBuilder, ReportKind};

use crate::analyze::{
    Error, Span,
    lex::token::{Keyword, Operator, Token},
};

pub mod token;

pub struct Lexer {
    source_name: Rc<String>,
    color_gen: ColorGenerator,
    code: Vec<char>,
    index: usize,
    last: Option<(Token, Range<usize>)>,
    current: Option<(Token, Range<usize>)>,
    next: Option<(Token, Range<usize>)>,
}

impl Lexer {
    pub fn new(source_name: Rc<String>, code: impl AsRef<str>) -> Result<Self, Error> {
        let code: Vec<char> = code.as_ref().chars().collect();

        let mut lexer = Self {
            source_name,
            color_gen: ColorGenerator::new(),
            code,
            index: 0,
            last: None,
            current: None,
            next: None,
        };

        lexer.lex_two()?;

        Ok(lexer)
    }

    pub fn cur_token_start(&self) -> usize {
        self.current
            .as_ref()
            .map(|(_, r)| r.start)
            .unwrap_or(self.index)
    }

    pub fn last_token_end(&self) -> usize {
        self.last.as_ref().map(|(_, r)| r.end).unwrap_or(self.index)
    }

    /// Get current token
    pub fn current(&self) -> Option<&(Token, Range<usize>)> {
        self.current.as_ref()
    }

    pub fn take_current(&mut self) -> Result<Option<(Token, Range<usize>)>, Error> {
        self.lex_one()?;
        Ok(self.last.clone())
    }

    /// Lookahead to next token
    pub fn next(&self) -> Option<&(Token, Range<usize>)> {
        self.next.as_ref()
    }

    /// Move on from current token to the next
    pub fn lex_one(&mut self) -> Result<(), Error> {
        self.last = self.current.take();
        self.current = self.next.take();
        self.next = self.lex_next()?;
        Ok(())
    }

    /// Move on and skip the next token
    pub fn lex_two(&mut self) -> Result<(), Error> {
        self.lex_one()?;
        self.lex_one()?;
        Ok(())
    }
}

/// Internals
impl Lexer {
    fn peek_char(&self) -> Option<char> {
        self.code.get(self.index + 1).copied()
    }

    fn cur_char(&self) -> Option<char> {
        self.code.get(self.index).copied()
    }

    fn find_next_lexable(&mut self) {
        while let Some(c) = self.cur_char() {
            if c == '/' && self.peek_char() == Some('/') {
                self.lex_comment();
            } else if c.is_whitespace() {
                self.index += 1;
            } else {
                break;
            }
        }
    }

    fn lex_next(&mut self) -> Result<Option<(Token, Range<usize>)>, Error> {
        self.find_next_lexable();

        let Some(c) = self.cur_char() else {
            return Ok(None);
        };

        if let Some((op, wide_op)) = Operator::parse(c, self.peek_char()) {
            self.index += if wide_op { 2 } else { 1 };
            return Ok(Some((Token::Operator(op), (self.index - 1)..self.index)));
        }

        if c.is_ascii_alphabetic() || c == '_' {
            return Ok(Some(self.lex_ascii()));
        }

        if c.is_ascii_digit() {
            return Ok(Some(self.lex_number()));
        }

        if c == ';' {
            self.index += 1;
            return Ok(Some((Token::Semicolon, (self.index - 1)..self.index)));
        }

        Err(Error::new(
            self.error(self.index, 1)
                .with_message("unexpected character")
                .with_code(3)
                .with_label(self.label(self.index, 1).with_message("what is this?"))
                .finish(),
        ))
    }

    fn lex_ascii(&mut self) -> (Token, Range<usize>) {
        let start = self.index;
        let mut string = String::new();
        while let Some(c) = self.cur_char()
            && (c.is_ascii_alphanumeric() || c == '_')
        {
            string.push(c);
            self.index += 1;
        }

        let token = if let Some(keyword) = Keyword::parse(&string) {
            Token::Keyword(keyword)
        } else {
            Token::Ident(string)
        };

        (token, start..self.index)
    }

    fn lex_number(&mut self) -> (Token, Range<usize>) {
        let start = self.index;
        let mut string = String::new();
        while let Some(c) = self.cur_char()
            && c.is_ascii_digit()
        {
            string.push(c);
            self.index += 1;
        }

        let num: i64 = string.parse().unwrap();
        (Token::Number(num), start..self.index)
    }

    fn lex_comment(&mut self) {
        while let Some(c) = self.peek_char() {
            self.index += 1;
            if c == '\n' {
                break;
            }
        }
    }

    fn error(&self, pos: usize, length: usize) -> ReportBuilder<'static, Span> {
        Report::build(
            ReportKind::Error,
            (self.source_name.clone(), pos..(pos + length)),
        )
    }

    fn label(&mut self, pos: usize, length: usize) -> Label<Span> {
        Label::new((self.source_name.clone(), pos..(pos + length)))
            .with_color(self.color_gen.next())
    }
}
