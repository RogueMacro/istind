use std::{
    fmt::Display,
    ops::{Deref, Range},
    rc::Rc,
};

use ariadne::{ColorGenerator, Label, Report, ReportBuilder, ReportKind, Source};

pub mod ast;
pub mod lex;
pub mod semantics;

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

pub struct ErrorBuilder<'c> {
    builder: ReportBuilder<'static, Span>,
    context: &'c mut ErrorContext,
}

impl<'c> ErrorBuilder<'c> {
    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.builder = self.builder.with_code(code);
        self
    }

    pub fn with_message(mut self, msg: impl ToString) -> Self {
        self.builder.set_message(msg);
        self
    }

    pub fn with_label(mut self, range: Range<usize>, msg: impl ToString) -> Self {
        let label = Label::new((self.context.source_name.clone(), range))
            // .with_color(ariadne::Color::BrightRed)
            .with_color(self.context.color_gen.next())
            .with_message(msg);

        self.builder.add_label(label);

        self
    }

    pub fn report(self) {
        let error = Error::new(self.builder.finish());
        self.context.errors.push(error);
    }

    pub fn finish(self) -> Error {
        Error::new(self.builder.finish())
    }
}

pub struct ErrorContext {
    source_name: Rc<String>,
    color_gen: ColorGenerator,
    errors: Vec<Error>,
}

impl ErrorContext {
    pub fn new(source_name: Rc<String>) -> Self {
        Self {
            source_name,
            color_gen: ColorGenerator::new(),
            errors: Vec::new(),
        }
    }

    pub fn unexpected_token(
        &mut self,
        range: Range<usize>,
        message: impl ToString,
    ) -> ErrorBuilder<'_> {
        self.build(range.clone())
            .with_code(ErrorCode::UnexpectedToken)
            .with_message("unexpected token")
            .with_label(range, message)
    }

    pub fn unexpected_eof(&mut self, pos: usize) -> ErrorBuilder<'_> {
        self.build((pos - 1)..pos)
            .with_code(ErrorCode::UnexpectedToken)
            .with_message("unexpected end of file")
            .with_label((pos - 1)..pos, "why stop here??")
    }

    pub fn build(&mut self, range: Range<usize>) -> ErrorBuilder<'_> {
        let builder = Report::build(ReportKind::Error, (self.source_name.clone(), range));

        ErrorBuilder {
            builder,
            context: self,
        }
    }

    pub fn report(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn take_errors(&mut self) -> ErrorVec {
        ErrorVec(std::mem::take(&mut self.errors))
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

pub struct ErrorVec(Vec<Error>);

impl ErrorVec {
    /// Prints all errors to stderr
    pub fn dump(&self, source_name: Rc<String>, source: &Source) {
        for error in &self.0 {
            error
                .eprint((source_name.clone(), source))
                .expect("couldn't print error message to stderr");

            eprintln!();
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Error> for ErrorVec {
    fn from(error: Error) -> Self {
        Self(vec![error])
    }
}
