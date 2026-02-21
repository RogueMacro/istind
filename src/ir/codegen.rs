use std::collections::HashMap;

use crate::{
    analyze::ast::{AST, Expression, Item as AstItem, Statement},
    ir::{BasicBlock, IR, Item, Op, SourceVal, VirtualReg},
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
    vregs: HashMap<String, VirtualReg>,
    vreg_counter: u32,
    ops: Vec<Op>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            vregs: HashMap::new(),
            vreg_counter: 0,
            ops: Vec::new(),
        }
    }

    pub fn build(mut self, block: Vec<Statement>) -> BasicBlock {
        self.ops.reserve(block.len());

        for stmt in block {
            match stmt {
                Statement::Declare { var, expr } => {
                    assert!(!self.vregs.contains_key(&var), "variable declared twice");

                    let dest = self.get_or_insert_vreg(var);
                    let src = self.unroll_expr(&expr, Some(dest));
                    self.ops.push(Op::Assign { src, dest });
                }
                Statement::Return(expr) => {
                    let value = self.unroll_expr(&expr, None);
                    self.ops.push(Op::Return { value });
                }
                Statement::FnCall(function) => {
                    self.ops.push(Op::Call { function });
                }
            }
        }

        BasicBlock { ops: self.ops }
    }

    fn unroll_expr(&mut self, expr: &Expression, dest: Option<VirtualReg>) -> SourceVal {
        match expr {
            Expression::Const(num) => SourceVal::Immediate(*num),
            Expression::Var(var) => SourceVal::VReg(self.get_or_insert_vreg(var)),
            Expression::Addition(expr1, expr2) => {
                let a = self.unroll_expr(expr1.as_ref(), None);
                let b = self.unroll_expr(expr2.as_ref(), None);

                let dest = dest.unwrap_or_else(|| self.get_vreg());
                self.ops.push(Op::Add { a, b, dest });

                SourceVal::VReg(dest)
            }
            Expression::Subtraction(expr1, expr2) => {
                let a = self.unroll_expr(expr1.as_ref(), None);
                let b = self.unroll_expr(expr2.as_ref(), None);

                let dest = dest.unwrap_or_else(|| self.get_vreg());
                self.ops.push(Op::Subtract { a, b, dest });

                SourceVal::VReg(dest)
            }
            Expression::Multiplication(expr1, expr2) => {
                let a = self.unroll_expr(expr1.as_ref(), None);
                let a = self.src_to_vreg(a);

                let b = self.unroll_expr(expr2.as_ref(), None);
                let b = self.src_to_vreg(b);

                let dest = dest.unwrap_or_else(|| self.get_vreg());
                self.ops.push(Op::Multiply { a, b, dest });

                SourceVal::VReg(dest)
            }
            Expression::Division(expr1, expr2) => {
                let a = self.unroll_expr(expr1.as_ref(), None);
                let b = self.unroll_expr(expr2.as_ref(), None);

                let dest = dest.unwrap_or_else(|| self.get_vreg());
                self.ops.push(Op::Divide { a, b, dest });

                SourceVal::VReg(dest)
            }
            Expression::FnCall(function) => todo!(),
        }
    }

    fn get_or_insert_vreg<S: Into<String> + AsRef<str>>(&mut self, var: S) -> VirtualReg {
        if let Some(&vreg) = self.vregs.get(var.as_ref()) {
            vreg
        } else {
            let vreg = self.get_vreg();
            self.vregs.insert(var.into(), vreg);
            vreg
        }
    }

    fn get_vreg(&mut self) -> VirtualReg {
        let vreg = VirtualReg(self.vreg_counter);
        self.vreg_counter += 1;
        vreg
    }

    fn src_to_vreg(&mut self, src: SourceVal) -> VirtualReg {
        match src {
            SourceVal::Immediate(_) => {
                let dest = self.get_vreg();
                self.ops.push(Op::Assign { src, dest });
                dest
            }
            SourceVal::VReg(vreg) => vreg,
        }
    }
}
