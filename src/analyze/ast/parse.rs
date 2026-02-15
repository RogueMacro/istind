use std::{ops::Range, rc::Rc};

use crate::analyze::{
    Error, ErrorCode, ErrorContext,
    ast::{AST, Expression, Item, Statement},
    lex::{
        Lexer,
        token::{Keyword, Operator, Token},
    },
};

pub struct Parser {
    err_ctx: ErrorContext,
    lexer: Lexer,
}

impl Parser {
    pub fn new(source_name: Rc<String>, lexer: Lexer) -> Self {
        Self {
            err_ctx: ErrorContext::new(source_name),
            lexer,
        }
    }

    pub fn into_ast(mut self) -> Result<AST, Error> {
        let mut ast = AST::new();

        while self.lexer.current().is_some() {
            let item = self.parse_item()?;
            ast.add_item(item);
        }

        Ok(ast)
    }

    fn parse_item(&mut self) -> Result<Item, Error> {
        let (token, range) = self.take_next()?;
        let Token::Keyword(keyword) = token else {
            return Err(self
                .err_ctx
                .unexpected_token(range, "expected keyword (fn)"));
        };

        match keyword {
            Keyword::Function => self.parse_function(),
            _ => Err(self.err_ctx.unexpected_token(range, "expected keyword: fn")),
        }
    }

    fn parse_function(&mut self) -> Result<Item, Error> {
        let (token, range) = self.take_next()?;
        let Token::Ident(name) = token else {
            return Err(self
                .err_ctx
                .unexpected_token(range, "expected function name"));
        };

        self.expect_next(
            |t| matches!(t, Token::Operator(Operator::LeftParenthesis)),
            "expected opening parenthesis",
        )?;

        self.expect_next(
            |t| matches!(t, Token::Operator(Operator::RightParenthesis)),
            "expected closing parenthesis",
        )?;

        let body = self.parse_body()?;

        Ok(Item::Function { name, body })
    }

    fn parse_body(&mut self) -> Result<Vec<Statement>, Error> {
        self.expect_next(
            |t| matches!(t, Token::Operator(Operator::LeftCurlyBracket)),
            "expected opening curly bracket",
        )?;

        let mut statements = Vec::new();
        while let Some((token, range)) = self.lexer.current() {
            if matches!(token, Token::Operator(Operator::RightCurlyBracket)) {
                self.lexer.take_current()?;
                return Ok(statements);
            }

            let statement = match token {
                Token::Keyword(_) => self.parse_keyword()?,
                _ => {
                    return Err(self
                        .err_ctx
                        .unexpected_token(range.clone(), "unknown identifier"));
                }
            };

            statements.push(statement);
        }

        Err(self.err_ctx.unexpected_eof(self.lexer.index()))
    }

    fn parse_keyword(&mut self) -> Result<Statement, Error> {
        let (Token::Keyword(keyword), range) = self.take_next()? else {
            unreachable!()
        };

        match keyword {
            Keyword::Return => self.parse_return(),
            Keyword::Let => self.parse_declaration(),
            _ => Err(self.err_ctx.unexpected_token(range, "unexpected keyword")),
        }
    }

    fn parse_return(&mut self) -> Result<Statement, Error> {
        let expr = self.parse_expr()?;
        self.expect_semicolon()?;

        Ok(Statement::Return(expr))
    }

    fn parse_declaration(&mut self) -> Result<Statement, Error> {
        let (token, range) = self.take_next()?;
        let Token::Ident(var) = token else {
            return Err(self
                .err_ctx
                .unexpected_token(range, "expected variable name"));
        };

        self.expect_next(
            |t| matches!(t, Token::Operator(Operator::Equality)),
            "expected equality operator",
        )?;

        let expr = self.parse_expr()?;
        self.expect_semicolon()?;

        Ok(Statement::Declare { var, expr })
    }

    fn parse_expr(&mut self) -> Result<Expression, Error> {
        let pos = self.lexer.index();
        let token = self.lexer.take_current()?;

        let expr = match token {
            Some((Token::Number(num), _)) => Expression::Const(num),
            Some((Token::Ident(ident), _)) => Expression::Var(ident),
            Some((token, range)) => {
                return Err(self.err_ctx.unexpected_token(
                    range,
                    format!("expected number after return, got: {:?}", token),
                ));
            }
            None => {
                return Err(self
                    .err_ctx
                    .unexpected_token((pos - 1)..pos, "unexpected end of file"));
            }
        };

        if matches!(
            self.lexer.current(),
            Some((Token::Operator(Operator::Plus), _))
        ) {
            self.lexer.take_current()?;
            let sub_expr = self.parse_expr()?;

            return Ok(Expression::Addition(Box::new(expr), Box::new(sub_expr)));
        }

        Ok(expr)
    }

    fn expect_next<F>(&mut self, matches: F, message: impl ToString) -> Result<Token, Error>
    where
        F: FnOnce(&Token) -> bool,
    {
        let (token, range) = self.take_next()?;
        if !matches(&token) {
            return Err(self.err_ctx.unexpected_token(range, message));
        }

        Ok(token)
    }

    fn expect_semicolon(&mut self) -> Result<(), Error> {
        let current = self.lexer.take_current()?;
        if matches!(current, Some((Token::Semicolon, _))) {
            Ok(())
        } else {
            let pos = current.map(|t| t.1.start).unwrap_or(self.lexer.index());
            Err(Error::new(
                self.err_ctx
                    .build(pos..(pos + 1))
                    .with_code(ErrorCode::MissingSemicolon)
                    .with_message("expected semicolon")
                    .with_label(
                        self.err_ctx
                            .label((pos - 1)..pos)
                            .with_message("insert the semicolon dummy"),
                    )
                    .finish(),
            ))
        }
    }

    fn take_next(&mut self) -> Result<(Token, Range<usize>), Error> {
        let token = self.lexer.take_current()?;
        match token {
            Some(token) => Ok(token),
            None => Err(self.err_ctx.unexpected_token(
                (self.lexer.index() - 1)..self.lexer.index(),
                "unexpected end of file",
            )),
        }
    }
}
