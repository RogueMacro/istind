use std::collections::{BTreeMap, HashMap};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use strum::EnumIter;
use ux::u12;

use crate::{
    ir::{BasicBlock, VirtualReg},
    synthesize::arch::arm::{
        ArmAssembler,
        instr::{self, Input},
    },
};

pub type Reg = Register;
pub type RegMap = HashMap<(VirtualReg, usize), RegisterGuard>;

#[repr(u32)]
#[derive(EnumIter, FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// (save to, load from, reg)
    SaveAndLoad(u12, u12, Register),
}

impl RegisterGuard {
    pub fn inner_reg(&self) -> Register {
        match *self {
            RegisterGuard::Ready(reg) => reg,
            RegisterGuard::Load(_, reg) => reg,
            RegisterGuard::Save(_, reg) => reg,
            RegisterGuard::SaveAndLoad(_, _, reg) => reg,
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
            Self::SaveAndLoad(save_to, load_from, dest) => {
                asm.emit(instr::Store {
                    base: Reg::SP,
                    offset: Input::Imm(save_to),
                    register: dest,
                });

                asm.emit(instr::Load {
                    stack_offset: load_from,
                    dest,
                });
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

/// Returns (RegMap, stack size)
pub fn allocate(bb: &BasicBlock) -> (RegMap, u12) {
    use Register::*;

    let mut phys_regs = vec![
        X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    ];

    let mut map: BTreeMap<VirtualReg, Location> = BTreeMap::new();
    let mut lifetimes = bb.lifetimes();
    let lifetimes_imm = lifetimes.clone();

    let last_uses: Vec<(VirtualReg, usize)> = lifetimes
        .iter()
        .map(|(vreg, lifetime)| (*vreg, lifetime.end().unwrap() - 1))
        .collect();

    let mut stack_ptr = u12::new(0);

    let mut regmap = RegMap::new();

    for (op_idx, _) in bb.ops.iter().enumerate() {
        println!(
            "\n> At instruction {} (%0: {:?})",
            op_idx,
            map.get(&VirtualReg(0))
        );
        let mut retired_regs = Vec::new();

        for (&vreg, lifetime) in lifetimes.iter_mut() {
            if let Some(interval) = lifetime.at_mut(op_idx) {
                println!("overlapping interval: {} (loc: {:?})", vreg, map.get(&vreg));
                // This vreg overlaps (is active) at this op index
                let reg_guard: RegisterGuard = match (interval.register, map.get(&vreg).copied()) {
                    (Some(reg), _) => {
                        // This interval has already been allocated.
                        println!("already allocated in reg X{}", reg);
                        interval.register = Some(reg);
                        RegisterGuard::Ready(Register::from_u32(reg).unwrap())
                    }
                    (None, Some(Location::Register(reg))) => {
                        // This value already exists in a register from a previous allocation.
                        println!("already in reg X{}", reg as u32);
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
                                println!("it is on the stack, found empty reg X{}", reg as u32);
                                interval.register = Some(reg as u32);
                                RegisterGuard::Load(offset, reg)
                            }
                            (Some(reg), None) => {
                                println!(
                                    "it is not on the stack, but found empty reg X{}",
                                    reg as u32
                                );
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
                                        println!(
                                            "it is on the stack, found dead reg X{} ({})",
                                            reg as u32, dead_vreg
                                        );
                                        retired_regs.push(dead_vreg);
                                        interval.register = Some(reg as u32);
                                        RegisterGuard::Load(offset, reg)
                                    }
                                    (Some((dead_vreg, reg)), None) => {
                                        println!(
                                            "it is NOT on the stack, but dead reg X{} ({})",
                                            reg as u32, dead_vreg
                                        );
                                        retired_regs.push(dead_vreg);
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

                                        map.insert(furthest_use_vreg, Location::Stack(stack_ptr));
                                        println!(
                                            "!added location on stack for {}",
                                            furthest_use_vreg
                                        );
                                        println!("now: {:?}", map.get(&furthest_use_vreg));
                                        interval.register = Some(reg as u32);

                                        let reg_guard = if let Some(offset) = location {
                                            println!(
                                                "it is on the stack, pushing reg {} (X{})",
                                                furthest_use_vreg, reg as u32
                                            );
                                            RegisterGuard::SaveAndLoad(stack_ptr, offset, reg)
                                        } else {
                                            println!(
                                                "it is NOT on the stack, but pushing reg {} (X{})",
                                                furthest_use_vreg, reg as u32
                                            );
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

                if let Some(l) = map.insert(vreg, Location::Register(reg_guard.inner_reg())) {
                    if let Location::Register(r) = l
                        && r == reg_guard.inner_reg()
                    {
                    } else {
                        println!("!replaced location for {} from {:?}", vreg, l);
                    }
                }
                regmap.insert((vreg, op_idx), reg_guard);
            }
        }

        for vreg in retired_regs {
            println!("/ retired {}", vreg);
            map.remove(&vreg);
        }
    }

    // TODO
    // difference between dest/src registers:
    // dead src reg can only be reused for a dest reg
    // on the same instruction, not another src reg.

    println!("Lifetimes ({}):", lifetimes.len());
    crate::ir::lifetime::print_lifetimes(&lifetimes);

    (regmap, stack_ptr)
}
