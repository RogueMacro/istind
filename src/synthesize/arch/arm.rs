use std::collections::HashMap;

use ux::{i12, i26, u12};

use crate::{
    ir::{IR, Item, Operation, SourceVal, VirtualReg},
    synthesize::arch::{
        Assemble, MachineCode,
        arm::{
            instr::{ImmShift16, Instruction},
            reg::{Reg, RegMap, Register},
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
    regmap: Option<RegMap>,
    stacks: Vec<i12>,
}

impl Assemble for ArmAssembler {
    fn assemble(ir: IR) -> MachineCode {
        let mut asm = ArmAssembler::default();

        for item in ir.items {
            asm.asm_item(item);
        }

        asm.code.entry_point_offset = asm.current_offset();
        let rel_main_offset =
            (*asm.functions.get("main").unwrap() as i32 - asm.current_offset() as i32) / 4;

        asm.emit(instr::BranchLink {
            addr: i26::new(rel_main_offset),
        });

        asm.emit(instr::Movz {
            shift: ImmShift16::L0,
            imm_value: 0x1,
            dest: Reg::X16,
        });

        asm.emit(instr::Syscall);

        asm.code
    }
}

impl ArmAssembler {
    fn emit(&mut self, instr: impl Instruction) {
        self.code.instructions.extend(instr.encode().to_le_bytes());
    }

    fn current_offset(&self) -> u64 {
        self.code.instructions.len() as u64
    }

    fn map_reg(&mut self, vreg: VirtualReg, op_idx: usize) -> Register {
        let reg_guard = *self
            .regmap
            .as_ref()
            .expect("no regmap generated for this operation")
            .get(&(vreg, op_idx))
            .expect(&format!(
                "no physical register mapped to {} at index={}",
                vreg, op_idx
            ));

        reg_guard.unwrap(self)
    }

    fn asm_item(&mut self, item: Item) {
        let Item::Function { name, bb } = item;
        self.functions.insert(name, self.current_offset());

        let (regmap, stack_size) = reg::allocate(&bb);
        self.regmap = Some(regmap);

        self.begin_stack(stack_size);

        for (idx, op) in bb.ops.iter().copied().enumerate() {
            self.asm_op(op, idx);
        }

        self.regmap = None;
    }

    fn begin_stack(&mut self, stack_size: u12) {
        if stack_size != u12::new(0) {
            let mut stack_size: u16 = stack_size.into();
            // align to 16 bytes
            if !stack_size.is_multiple_of(2) {
                stack_size += 1;
            }

            let stack_size = stack_size as i16;
            let stack_size = i12::new(stack_size * 8);

            println!("pushing stack: {}", stack_size);
            self.stacks.push(stack_size);

            self.emit(instr::Sub {
                imm: stack_size,
                src: Reg::SP,
                dest: Reg::SP,
            });
        }
    }

    fn end_stack(&mut self) {
        if let Some(stack_size) = self.stacks.pop()
            && stack_size != i12::new(0)
        {
            self.emit(instr::Add {
                a: Reg::SP,
                b: instr::Input::Imm(stack_size),
                dest: Reg::SP,
            });
        }
    }

    fn asm_op(&mut self, op: Operation, idx: usize) {
        match op {
            Operation::Assign { src, dest } => self.emit_assign(src, dest, idx),
            Operation::Add { a, b, dest } => self.emit_add(a, b, dest, idx),
            Operation::Return { value } => self.emit_return(value, idx),
        }
    }

    fn emit_store(&mut self, offset: u12, register: Register) {
        self.emit(instr::Store {
            base: Reg::SP,
            offset: instr::Input::Imm(offset),
            register,
        });
    }

    fn emit_add(&mut self, a: SourceVal, b: SourceVal, dest: VirtualReg, idx: usize) {
        let dest = self.map_reg(dest, idx);
        match (a, b) {
            (SourceVal::Immediate(a), SourceVal::Immediate(b)) => self.emit_movz(a + b, dest),
            (SourceVal::Immediate(n), SourceVal::VReg(vreg))
            | (SourceVal::VReg(vreg), SourceVal::Immediate(n)) => {
                assert!(n <= i16::MAX as i64);
                let a = self.map_reg(vreg, idx);
                self.emit(instr::Add {
                    a,
                    b: instr::Input::Imm(i12::new(n as i16)),
                    dest,
                });
            }
            (SourceVal::VReg(a), SourceVal::VReg(b)) => {
                let a = self.map_reg(a, idx);
                let b = self.map_reg(b, idx);
                self.emit(instr::Add {
                    a,
                    b: instr::Input::Reg(b),
                    dest,
                })
            }
        }
    }

    fn emit_return(&mut self, src: SourceVal, idx: usize) {
        match src {
            SourceVal::Immediate(n) => self.emit_movz(n, Reg::X0),
            SourceVal::VReg(vreg) => {
                let src = self.map_reg(vreg, idx);
                self.emit(instr::MovReg { src, dest: Reg::X0 });
            }
        }

        self.end_stack();
        self.emit(instr::Ret);
    }

    fn emit_movz(&mut self, n: i64, dest: Register) {
        self.emit(instr::Movz {
            shift: ImmShift16::L0,
            imm_value: n as u16,
            dest,
        });
    }

    fn emit_assign(&mut self, src: SourceVal, dest: VirtualReg, idx: usize) {
        let dest = self.map_reg(dest, idx);
        match src {
            SourceVal::Immediate(n) => self.emit_movz(n, dest),
            SourceVal::VReg(vreg) => {
                let src = self.map_reg(vreg, idx);
                self.emit(instr::MovReg { src, dest });
            }
        }
    }
}
