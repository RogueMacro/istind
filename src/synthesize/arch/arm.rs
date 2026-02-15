use std::collections::HashMap;

use strum::IntoEnumIterator;
use ux::i26;

use crate::{
    ir::{IR, Item, Operation, SourceVal, VirtualReg},
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
    var_regs: HashMap<u32, Register>,
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
        println!("Assembling function {}", name);
        self.functions.insert(name, self.current_offset());

        for op in bb.ops {
            self.asm_op(op);
        }

        self.var_regs.clear();
        self.ready_regs.clear();
        self.ready_regs.extend(Register::iter().take(15));
    }

    fn asm_op(&mut self, op: Operation) {
        // TODO
        // match op {
        //     Operation::Store {
        //         stack_offset,
        //         value,
        //     } => self.emit_store(stack_offset, value),
        //     Operation::Add { a, b, dest } => self.emit_add(a, b, dest),
        //     Operation::Return { value } => self.emit_return(value),
        // }
    }

    fn emit_add(&mut self, a: SourceVal, b: SourceVal, dest: VirtualReg) {
        // let reg_a = self.use_value(a);
        // let reg_b = self.use_value(b);
        // let dest_reg = self.ready_regs.pop().unwrap();
        //
        // self.emit(instr::add_reg(reg_a, reg_b, dest_reg));
        //
        // match dest {
        //     DestVal::Temporary(tmp) => {
        //         self.tmp_regs.insert(tmp, dest_reg);
        //     }
        //     DestVal::Stack(_) => todo!(), // move value to the stack and free the dest_reg to
        //                                   // ready_regs
        // }
    }

    fn emit_return(&mut self, value: SourceVal) {
        // self.put_value(value, Reg::X0);
        // self.emit(instr::Ret);
    }
}
