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
        let (token, range) = self.expect_take_current()?;
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
        let (token, range) = self.expect_take_current()?;
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

        let body = self.parse_block()?;

        Ok(Item::Function { name, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, Error> {
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

            let range = range.clone();

            if let Token::Keyword(keyword) = token {
                let keyword = *keyword;
                self.lexer.take_current()?;
                let stmt = self.parse_keyword(keyword, range)?;
                statements.push(stmt);
            } else if let Expression::FnCall(function) = self.parse_expr()? {
                self.expect_semicolon()?;
                statements.push(Statement::FnCall(function));
            } else {
                return Err(self
                    .err_ctx
                    .unexpected_token(range.clone(), "this is not in our dictionary"));
            }
        }

        Err(self.err_ctx.unexpected_eof(self.lexer.index()))
    }

    fn parse_keyword(&mut self, keyword: Keyword, range: Range<usize>) -> Result<Statement, Error> {
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
        let (token, range) = self.expect_take_current()?;
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
            Some((Token::Ident(ident), _)) => self.parse_ident_expr(ident)?,
            Some((_, range)) => {
                return Err(self.err_ctx.unexpected_token(range, "invalid expression"));
            }
            None => {
                return Err(self
                    .err_ctx
                    .unexpected_token((pos - 1)..pos, "unexpected end of file"));
            }
        };

        if let Some((Token::Operator(op), _)) = self.lexer.current()
            && matches!(
                op,
                Operator::Plus | Operator::Minus | Operator::Star | Operator::Slash
            )
        {
            // Better way to pattern match and avoid shadowing?
            let op = *op;

            self.lexer.take_current()?;
            let sub_expr = self.parse_expr()?;

            let expr = match op {
                Operator::Plus => Expression::Addition(Box::new(expr), Box::new(sub_expr)),
                Operator::Minus => Expression::Subtraction(Box::new(expr), Box::new(sub_expr)),
                Operator::Star => Expression::Multiplication(Box::new(expr), Box::new(sub_expr)),
                Operator::Slash => Expression::Division(Box::new(expr), Box::new(sub_expr)),
                _ => unreachable!(),
            };

            return Ok(expr);
        }

        Ok(expr)
    }

    fn parse_ident_expr(&mut self, ident: String) -> Result<Expression, Error> {
        if matches!(
            self.lexer.current(),
            Some((Token::Operator(Operator::LeftParenthesis), _))
        ) {
            self.lexer.take_current()?;

            self.expect_next(
                |t| matches!(t, Token::Operator(Operator::RightParenthesis)),
                "expected closing parenthesis",
            )?;

            Ok(Expression::FnCall(ident))
        } else {
            Ok(Expression::Var(ident))
        }
    }

    fn expect_next<F>(&mut self, matches: F, message: impl ToString) -> Result<Token, Error>
    where
        F: FnOnce(&Token) -> bool,
    {
        let (token, range) = self.expect_take_current()?;
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

    fn expect_take_current(&mut self) -> Result<(Token, Range<usize>), Error> {
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
