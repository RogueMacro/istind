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
            current: None,
            next: None,
        };

        lexer.lex_two()?;

        Ok(lexer)
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Get current token
    pub fn current(&self) -> Option<&(Token, Range<usize>)> {
        self.current.as_ref()
    }

    pub fn take_current(&mut self) -> Result<Option<(Token, Range<usize>)>, Error> {
        let cur = self.current.take();
        self.lex_one()?;
        Ok(cur)
    }

    /// Lookahead to next token
    pub fn next(&self) -> Option<&(Token, Range<usize>)> {
        self.next.as_ref()
    }

    /// Move on from current token to the next
    pub fn lex_one(&mut self) -> Result<(), Error> {
        self.current = self.next.take();
        self.next = self.lex_next()?;
        if let Some((tok, _)) = &self.next {
            println!("lexed: {:?}", tok);
        }
        Ok(())
    }

    /// Move on and skip the next token
    pub fn lex_two(&mut self) -> Result<(), Error> {
        // self.current = self.lex_next()?;
        // self.next = self.lex_next()?;
        self.lex_one()?;
        self.lex_one()?;
        Ok(())
    }
}

/// Internals
impl Lexer {
    fn peek_char(&self, offset: usize) -> Option<char> {
        self.code.get(self.index + offset).copied()
    }

    fn cur_char(&self) -> Option<char> {
        self.peek_char(0)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.cur_char()
            && c.is_whitespace()
        {
            self.index += 1;
        }
    }

    fn lex_next(&mut self) -> Result<Option<(Token, Range<usize>)>, Error> {
        self.skip_whitespace();

        let Some(c) = self.cur_char() else {
            return Ok(None);
        };

        if let Some(op) = Operator::parse(c) {
            self.index += 1;
            return Ok(Some((Token::Operator(op), (self.index - 1)..self.index)));
        }

        if c.is_ascii_alphabetic() {
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
            && c.is_ascii_alphanumeric()
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
