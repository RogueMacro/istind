use std::collections::{BTreeMap, HashMap};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use strum::EnumIter;
use ux::u12;

use crate::{
    ir::{BasicBlock, VirtualReg, lifetime::Interval},
    synthesize::arch::arm::{
        ArmAssembler,
        instr::{self, Input},
        reg,
    },
};

pub type Reg = Register;
pub type RegMap = HashMap<(VirtualReg, usize), RegisterGuard>;

#[repr(u32)]
#[derive(EnumIter, FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum Register {
    X0 = 0,   // 1st argument / return value
    X1 = 1,   // 2nd argument
    X2 = 2,   // 3rd argument
    X3 = 3,   // 4th argument
    X4 = 4,   // 5th argument
    X5 = 5,   // 6th argument
    X6 = 6,   // 7th argument
    X7 = 7,   // 8th argument
    X8 = 8,   // indirect result
    X9 = 9,   // caller-saved
    X10 = 10, // caller-saved
    X11 = 11, // caller-saved
    X12 = 12, // caller-saved
    X13 = 13, // caller-saved
    X14 = 14, // caller-saved
    X15 = 15, // caller-saved
    X16 = 16, // IP0
    X17 = 17, // IP1
    X18 = 18, // platform register
    X19 = 19, // callee-saved
    X20 = 20, // callee-saved
    X21 = 21, // callee-saved
    X22 = 22, // callee-saved
    X23 = 23, // callee-saved
    X24 = 24, // callee-saved
    X25 = 25, // callee-saved
    X26 = 26, // callee-saved
    X27 = 27, // callee-saved
    X28 = 28, // callee-saved
    FP = 29,  // frame pointer (X29)
    LR = 30,  // link register (X30)
    SP = 31,  // stack pointer (X31) (not general purpose)
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterGuard {
    Ready(Register),
    Load(u12, Register),
    Save(u12, Register),
    SaveAndLoad(u12, Register),
}

impl RegisterGuard {
    pub fn inner_reg(&self) -> Register {
        match *self {
            RegisterGuard::Ready(reg) => reg,
            RegisterGuard::Load(_, reg) => reg,
            RegisterGuard::Save(_, reg) => reg,
            RegisterGuard::SaveAndLoad(_, reg) => reg,
        }
    }

    pub fn unwrap(&self, asm: &mut ArmAssembler) -> Register {
        match *self {
            Self::Ready(reg) => reg,
            Self::Load(stack_offset, dest) => {
                asm.emit(instr::Load { stack_offset, dest });
                dest
            }
            Self::Save(stack_offset, dest) => {
                asm.emit_store(stack_offset, dest);
                dest
            }
            Self::SaveAndLoad(stack_offset, dest) => {
                asm.emit(instr::Store {
                    base: Reg::SP,
                    offset: Input::Imm(stack_offset),
                    register: dest,
                });

                asm.emit(instr::Load { stack_offset, dest });
                dest
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Register(Register),
    Stack(u12),
}

pub fn allocate(bb: &BasicBlock) -> RegMap {
    let mut phys_regs = vec![Reg::X0, Reg::X1, Reg::X2, Reg::X3];

    let mut map: BTreeMap<VirtualReg, Location> = BTreeMap::new();
    let mut lifetimes = bb.lifetimes();
    let lifetimes_imm = lifetimes.clone();

    let last_uses: Vec<(VirtualReg, usize)> = lifetimes
        .iter()
        .map(|(vreg, lifetime)| (*vreg, lifetime.end().unwrap() - 1))
        .collect();

    let mut stack = Vec::<bool>::new();
    let mut stack_ptr = u12::new(0);

    let mut regmap = RegMap::new();

    for (op_idx, _) in bb.ops.iter().enumerate() {
        for (&vreg, lifetime) in lifetimes.iter_mut() {
            if let Some(interval) = lifetime.at_mut(op_idx) {
                // This vreg overlaps (is active) at this op index
                let reg_guard: RegisterGuard = match (interval.register, map.get(&vreg).copied()) {
                    (Some(reg), _) => {
                        // This interval has already been allocated.
                        interval.register = Some(reg);
                        RegisterGuard::Ready(Register::from_u32(reg).unwrap())
                    }
                    (None, Some(Location::Register(reg))) => {
                        // This value already exists in a register from a previous allocation.
                        interval.register = Some(reg as u32);
                        RegisterGuard::Ready(reg)
                    }

                    (None, location) => {
                        let location = location.map(|l| {
                            let Location::Stack(offset) = l else {
                                unreachable!()
                            };
                            offset
                        });

                        match (phys_regs.pop(), location) {
                            (Some(reg), Some(offset)) => {
                                interval.register = Some(reg as u32);
                                RegisterGuard::Load(offset, reg)
                            }
                            (Some(reg), None) => {
                                interval.register = Some(reg as u32);
                                RegisterGuard::Ready(reg)
                            }
                            (None, location) => {
                                // All physical registers busy, look for dead virtual registers before pushing
                                // one onto the stack.

                                let mut dead_vreg = None;

                                for (vreg, loc) in map.iter() {
                                    if let Location::Register(reg) = loc {
                                        let (_, end) =
                                            last_uses.iter().find(|(v, _)| v == vreg).unwrap();
                                        if *end <= op_idx {
                                            dead_vreg = Some((*vreg, *reg));
                                            break;
                                        }
                                    }
                                }

                                match (dead_vreg, location) {
                                    (Some((dead_vreg, reg)), Some(offset)) => {
                                        map.remove(&dead_vreg);
                                        interval.register = Some(reg as u32);
                                        RegisterGuard::Load(offset, reg)
                                    }
                                    (Some((dead_vreg, reg)), None) => {
                                        map.remove(&dead_vreg);
                                        interval.register = Some(reg as u32);
                                        RegisterGuard::Ready(reg)
                                    }
                                    (None, location) => {
                                        // All busy virtual registers are to be used in the future.
                                        // Therefor we must save one register to the stack (preferrably the one to be
                                        // used last) that is not currently overlapping.

                                        let (&furthest_use_vreg, &reg, _) = map
                                            .iter()
                                            .filter_map(|(vreg, loc)| {
                                                if let Location::Register(reg) = loc {
                                                    let next_use = lifetimes_imm
                                                        .get(vreg)
                                                        .unwrap()
                                                        .next_use_after(op_idx)
                                                        .unwrap_or(usize::MAX);

                                                    Some((vreg, reg, next_use))
                                                } else {
                                                    None
                                                }
                                            })
                                            .max_by_key(|(_, _, next_use)| *next_use)
                                            .unwrap();

                                        map.remove(&furthest_use_vreg);
                                        interval.register = Some(reg as u32);

                                        let reg_guard = if let Some(offset) = location {
                                            RegisterGuard::SaveAndLoad(stack_ptr, reg)
                                        } else {
                                            RegisterGuard::Save(stack_ptr, reg)
                                        };

                                        stack_ptr = stack_ptr + u12::new(1);

                                        reg_guard
                                    }
                                }
                            }
                        }
                    }
                };

                map.insert(vreg, Location::Register(reg_guard.inner_reg()));
                regmap.insert((vreg, op_idx), reg_guard);
            }
        }
    }

    // TODO
    // difference between dest/src registers:
    // dead src reg can only be reused for a dest reg
    // on the same instruction, not another src reg.

    println!("Lifetimes ({}):", lifetimes.len());
    crate::ir::lifetime::print_lifetimes(&lifetimes);

    regmap
}
