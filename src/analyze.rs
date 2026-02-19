use std::{
    fmt::Display,
    ops::{Deref, Range},
    rc::Rc,
};

use ariadne::{ColorGenerator, Label, Report, ReportBuilder, ReportKind};

pub mod ast;
pub mod lex;

pub type Span = (Rc<String>, Range<usize>);

#[derive(Debug)]
pub struct Error(Box<Report<'static, Span>>);

impl Error {
    pub fn new(report: Report<'static, Span>) -> Self {
        Self(Box::new(report))
    }
}

impl Deref for Error {
    type Target = Report<'static, Span>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct ErrorContext {
    source_name: Rc<String>,
    color_gen: ColorGenerator,
}

impl ErrorContext {
    pub fn new(source_name: Rc<String>) -> Self {
        Self {
            source_name,
            color_gen: ColorGenerator::new(),
        }
    }

    pub fn unexpected_token(&mut self, range: Range<usize>, message: impl ToString) -> Error {
        Error::new(
            self.build(range.clone())
                .with_code(ErrorCode::UnexpectedToken)
                .with_message("unexpected token")
                .with_label(self.label(range).with_message(message))
                .finish(),
        )
    }

    pub fn unexpected_eof(&mut self, pos: usize) -> Error {
        Error::new(
            self.build((pos - 1)..pos)
                .with_code(ErrorCode::UnexpectedToken)
                .with_message("unexpected end of file")
                .with_label(self.label((pos - 1)..pos).with_message("why stop here??"))
                .finish(),
        )
    }

    pub fn build(&self, range: Range<usize>) -> ReportBuilder<'static, Span> {
        Report::build(ReportKind::Error, (self.source_name.clone(), range))
    }

    pub fn label(&mut self, range: Range<usize>) -> Label<Span> {
        Label::new((self.source_name.clone(), range)).with_color(self.color_gen.next())
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorCode {
    MissingSemicolon,
    UnexpectedToken,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:02}", *self as u32)
    }
}
