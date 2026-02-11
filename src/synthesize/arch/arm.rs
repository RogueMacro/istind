use std::collections::HashMap;

use strum::IntoEnumIterator;
use ux::{i26, u12};

use crate::{
    ir::{DestVal, IR, Item, Op, Operation, SourceVal, StackOffset},
    synthesize::arch::{
        Assemble, MachineCode,
        arm::{
            instr::{ImmShift, Instruction},
            reg::{Reg, Register},
        },
    },
};

pub mod instr;
pub mod reg;

const MAX_EXIT_CODE: u16 = 255; // On UNIX

#[derive(Default)]
pub struct ArmAssembler {
    code: MachineCode,
    functions: HashMap<String, u64>,
    var_regs: HashMap<StackOffset, Register>,
    tmp_regs: HashMap<u32, Register>,
    ready_regs: Vec<Register>,
}

impl Assemble for ArmAssembler {
    fn assemble(mut self, ir: IR) -> MachineCode {
        for item in ir.items {
            self.asm_item(item);
        }

        self.code.entry_point_offset = self.current_offset();
        let rel_main_offset =
            (*self.functions.get("main").unwrap() as i32 - self.current_offset() as i32) / 4;

        self.emit(instr::BranchLink {
            addr: i26::new(rel_main_offset),
        });

        self.emit(instr::Movz {
            shift: ImmShift::L0,
            imm_value: 0x1,
            dest_reg: Reg::X16,
        });

        self.emit(instr::Syscall);

        self.code
    }
}

impl ArmAssembler {
    pub fn new() -> Self {
        Self {
            code: MachineCode::new(),
            functions: HashMap::new(),
            var_regs: HashMap::new(),
            tmp_regs: HashMap::new(),
            ready_regs: Register::iter().take(15).collect(),
        }
    }

    fn emit(&mut self, instr: impl Instruction) {
        self.code.instructions.extend(instr.encode().to_le_bytes());
        println!("Emit: {:?}", instr);
    }

    fn current_offset(&self) -> u64 {
        self.code.instructions.len() as u64
    }

    fn asm_item(&mut self, item: Item) {
        let Item::Function { name, bb } = item;
        println!("Assembling {}", name);
        self.functions.insert(name, self.current_offset());

        for op in bb.ops {
            self.asm_op(op);
        }

        self.var_regs.clear();
        self.ready_regs.clear();
        self.ready_regs.extend(Register::iter().take(15));
    }

    fn asm_op(&mut self, op: Operation) {
        match op {
            Operation::Store {
                stack_offset,
                value,
            } => self.emit_store(stack_offset, value),
            Operation::Add { a, b, dest } => self.emit_add(a, b, dest),
            Operation::Return { value } => self.emit_return(value),
        }
    }

    fn emit_store(&mut self, stack_offset: StackOffset, value: SourceVal) {
        let value = self.use_value(value);
        let base = Register::SP;
        // let offset = self.use_value(SourceVal::Immediate(stack_offset as i64));

        self.emit(instr::Store {
            base,
            offset: instr::Input::Imm(u12::new(stack_offset as u16)),
            value,
        });
    }

    fn emit_add(&mut self, a: SourceVal, b: SourceVal, dest: DestVal) {
        let reg_a = self.use_value(a);
        let reg_b = self.use_value(b);
        let dest_reg = self.ready_regs.pop().unwrap();

        self.emit(instr::add_reg(reg_a, reg_b, dest_reg));

        match dest {
            DestVal::Temporary(tmp) => {
                self.tmp_regs.insert(tmp, dest_reg);
            }
            DestVal::Stack(_) => todo!(), // move value to the stack and free the dest_reg to
                                          // ready_regs
        }
    }

    fn emit_return(&mut self, value: SourceVal) {
        self.put_value(value, Reg::X0);
        self.emit(instr::Ret);
    }

    fn use_value(&mut self, value: SourceVal) -> Register {
        match value {
            SourceVal::Immediate(num) => {
                let reg = self.ready_regs.pop().unwrap();
                self.emit(instr::Movz {
                    shift: ImmShift::L0,
                    imm_value: num as u16,
                    dest_reg: reg,
                });
                reg
            }
            SourceVal::Temporary(tmp) => self.tmp_regs.get(&tmp).copied().unwrap(),
            SourceVal::Stack(stack_offset) => self
                .var_regs
                .get(&stack_offset)
                .copied()
                .unwrap_or_else(|| {
                    let reg = self.ready_regs.pop().unwrap();
                    self.put_value(value, reg);
                    reg
                }),
        }
    }

    fn put_value(&mut self, value: SourceVal, dest: Register) {
        match value {
            SourceVal::Immediate(num) => {
                self.emit(instr::Movz {
                    shift: ImmShift::L0,
                    imm_value: num as u16,
                    dest_reg: dest,
                });
            }
            SourceVal::Temporary(tmp) => {
                let src = self.tmp_regs.get(&tmp).copied().unwrap();
                if src != dest {
                    self.emit(instr::MovReg { src, dest });
                    self.tmp_regs.insert(tmp, dest);
                    self.ready_regs.push(src);
                }
            }
            SourceVal::Stack(stack_offset) => {
                if let Some(&src) = self.var_regs.get(&stack_offset) {
                    if src != dest {
                        self.emit(instr::MovReg { src, dest });
                        self.var_regs.insert(stack_offset, dest);
                        self.ready_regs.push(src);
                    }
                } else {
                    self.emit(instr::Load {
                        stack_offset: u12::new(stack_offset as u16),
                        dest,
                    })
                }
            }
        }
    }
}
