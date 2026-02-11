use std::collections::HashMap;

use crate::{
    analyze::ast::{AST, Expression, Item as AstItem, Statement},
    ir::{BasicBlock, DestVal, IR, Item, Lifetime, Op, SourceVal, StackOffset},
};

impl IR {
    pub fn generate(ast: AST) -> IR {
        let mut items = Vec::new();

        for item in ast.items {
            let item = match item {
                AstItem::Function { name, body } => Item::Function {
                    name,
                    bb: BlockBuilder::new().build(body),
                },
            };

            items.push(item);
        }

        IR { items }
    }
}

struct BlockBuilder {
    stack: HashMap<String, StackOffset>,
    stack_size: u32,
    ops: Vec<Op>,
    lifetimes: HashMap<u32, Lifetime>,
    tmp_var_counter: u32,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            stack: HashMap::new(),
            stack_size: 0,
            ops: Vec::new(),
            lifetimes: HashMap::new(),
            tmp_var_counter: 0,
        }
    }

    pub fn build(mut self, block: Vec<Statement>) -> BasicBlock {
        self.ops.reserve(block.len());

        for stmt in block {
            match stmt {
                Statement::Declare { var, expr } => {
                    let stack_offset = self.get_or_insert_stack(&var);

                    let value = self.unroll_expr(&expr);
                    self.ops.push(Op::Store {
                        stack_offset,
                        value,
                    });
                }
                Statement::Return(expr) => {
                    let value = self.unroll_expr(&expr);
                    self.ops.push(Op::Return { value });
                }
            }
        }

        BasicBlock {
            stack_size: self.stack_size,
            ops: self.ops,
            lifetimes: self.lifetimes,
        }
    }

    fn unroll_expr(&mut self, expr: &Expression) -> SourceVal {
        match expr {
            Expression::Const(num) => SourceVal::Immediate(*num),
            Expression::Var(var) => SourceVal::Stack(self.get_or_insert_stack(var)),
            Expression::Addition(expr1, expr2) => {
                let a = self.unroll_expr(expr1.as_ref());
                let b = self.unroll_expr(expr2.as_ref());

                let tmp = self.tmp_var_counter;
                self.tmp_var_counter += 1;
                self.ops.push(Op::Add {
                    a,
                    b,
                    dest: DestVal::Temporary(tmp),
                });

                SourceVal::Temporary(tmp)
            }
        }
    }

    fn get_or_insert_stack(&mut self, var: &str) -> StackOffset {
        self.stack.get(var).copied().unwrap_or_else(|| {
            self.stack.insert(var.to_owned(), self.stack_size);
            self.stack_size += 8; // TODO: var size
            self.stack_size - 8
        })
    }
}
