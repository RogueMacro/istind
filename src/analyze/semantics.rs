use std::{collections::HashMap, ops::Range, rc::Rc};

use crate::analyze::{
    ErrorContext, ErrorVec,
    ast::{AST, ExprType, Expression, Item, Statement},
};

pub struct ValidAST(pub AST);

const MAIN_FN: &str = "main";

pub fn analyze(ast: AST, source_name: Rc<String>) -> Result<ValidAST, ErrorVec> {
    let mut analyzer = Analyzer::new(&ast, source_name);
    analyzer.analyze()?;

    Ok(ValidAST(ast))
}

struct Analyzer<'ast> {
    ast: &'ast AST,
    err_ctx: ErrorContext,

    symbols: HashMap<String, Symbol>,
}

impl<'ast> Analyzer<'ast> {
    pub fn new(ast: &'ast AST, source_name: Rc<String>) -> Self {
        Self {
            ast,
            err_ctx: ErrorContext::new(source_name),
            symbols: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<(), ErrorVec> {
        for item in &self.ast.items {
            self.item(item);
        }

        if !self.err_ctx.is_empty() {
            return Err(self.err_ctx.take_errors());
        }

        Ok(())
    }

    fn item(&mut self, item: &Item) {
        let Item::Function {
            name,
            body,
            decl_range,
        } = item;

        if self
            .symbols
            .insert(name.clone(), Symbol::Function)
            .is_some()
        {
            self.err_ctx
                .build(decl_range.clone())
                .with_message("duplicate function definition")
                .with_label(decl_range.clone(), "already defined")
                .report();
        }

        let mut has_return = false;
        for stmt in body {
            if matches!(stmt, Statement::Return(_)) {
                has_return = true;
            }

            self.statement(stmt);
        }

        if !has_return && name == MAIN_FN {
            self.err_ctx
                .build(decl_range.clone())
                .with_message("no return statement found in function main")
                .with_label(decl_range.clone(), "main must return a value")
                .report();
        }
    }

    fn statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Declare {
                var,
                expr,
                var_range,
            } => {
                if self.symbols.insert(var.clone(), Symbol::Variable).is_some() {
                    self.err_ctx
                        .build(var_range.clone())
                        .with_message("duplicate variable declaration")
                        .with_label(var_range.clone(), "variable already defined")
                        .report();
                }

                self.expression(expr);
            }
            Statement::Assign {
                var,
                expr,
                var_range,
            } => {
                self.check_var(var, var_range);
                self.expression(expr);
            }
            Statement::Return(expr) => self.expression(expr),
            Statement::Expr(expr) => self.expression(expr),
            Statement::FnCall(_) => (),
        }
    }

    fn expression(&mut self, expr: &Expression) {
        match &expr.expr_type {
            ExprType::Const(_) => (),
            ExprType::Variable(var) => self.check_var(var, &expr.range),
            ExprType::Addition(expr1, expr2) => {
                self.expression(expr1);
                self.expression(expr2);
            }
            ExprType::Subtraction(expr1, expr2) => {
                self.expression(expr1);
                self.expression(expr2);
            }
            ExprType::Multiplication(expr1, expr2) => {
                self.expression(expr1);
                self.expression(expr2);
            }
            ExprType::Division(expr1, expr2) => {
                self.expression(expr1);
                self.expression(expr2);
            }
            ExprType::FnCall(_) => todo!(),
        }
    }

    fn check_var(&mut self, symbol: &str, range: &Range<usize>) {
        if !self.symbols.contains_key(symbol) {
            self.err_ctx
                .build(range.clone())
                .with_message("undeclared variable")
                .with_label(range.clone(), "this guy doesn't exist")
                .report();
        }
    }
}

enum Symbol {
    Variable,
    Function,
}
