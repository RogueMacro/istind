use std::ops::Range;

pub mod parse;

#[derive(Default, Debug)]
pub struct AST {
    pub items: Vec<Item>,
}

impl AST {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

#[derive(Debug)]
pub enum Item {
    Function {
        name: String,
        body: Vec<Statement>,
        decl_range: Range<usize>,
    },
}

#[derive(Debug)]
pub enum Statement {
    Declare {
        var: String,
        expr: Expression,
        var_range: Range<usize>,
    },
    Assign {
        var: String,
        expr: Expression,
        var_range: Range<usize>,
    },
    Return(Expression),
    Expr(Expression),
    FnCall(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Const(i64),
    Variable(String),
    Addition(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),

    FnCall(String),
}
