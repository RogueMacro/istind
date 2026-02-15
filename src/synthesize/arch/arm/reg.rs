use std::collections::{BTreeMap, HashMap};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use strum::EnumIter;
use ux::u12;

use crate::{
    ir::{
        BasicBlock, VirtualReg,
        lifetime::{Interval, Location},
    },
    synthesize::arch::arm::{
        ArmAssembler,
        instr::{self, Input},
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

#[derive(Debug)]
pub enum RegisterGuard {
    Ready(Register),
    Load(u12, Register),
    SaveAndLoad(u12, Register),
}

impl RegisterGuard {
    pub fn unwrap(&self, asm: &mut ArmAssembler) -> Register {
        match *self {
            Self::Ready(reg) => reg,
            Self::Load(stack_offset, dest) => {
                asm.emit(instr::Load { stack_offset, dest });
                dest
            }
            Self::SaveAndLoad(stack_offset, dest) => {
                asm.emit(instr::Store {
                    base: Reg::SP,
                    offset: Input::Imm(stack_offset),
                    value: dest,
                });

                asm.emit(instr::Load { stack_offset, dest });
                dest
            }
        }
    }
}

pub fn allocate(bb: &BasicBlock) -> RegMap {
    let mut phys_regs = vec![Reg::X0, Reg::X1, Reg::X2, Reg::X3];

    let mut map: BTreeMap<VirtualReg, Location> = BTreeMap::new();
    let mut lifetimes = bb.lifetimes();
    let last_uses: Vec<(VirtualReg, usize)> = lifetimes
        .iter()
        .map(|(vreg, lifetime)| (*vreg, lifetime.end().unwrap() - 1))
        .collect();

    let mut stack = Vec::<bool>::new();
    let mut stack_ptr = 0;

    let mut regmap = RegMap::new();

    for (op_idx, _) in bb.ops.iter().enumerate() {
        let mut overlapping = Vec::new();
        for (&vreg, lifetime) in lifetimes.iter_mut() {
            if let Some(interval) = lifetime.at_mut(op_idx) {
                if let Some(location) = interval.location {
                    match location {
                        Location::Register(r) => {
                            regmap.insert(
                                (vreg, op_idx),
                                RegisterGuard::Ready(Register::from_u32(r).unwrap()),
                            );
                        }
                        Location::Stack(offset) => {
                            panic!("why are we here?");
                        }
                    }
                } else if let Some(Location::Register(preg)) = map.get(&vreg) {
                    regmap.insert(
                        (vreg, op_idx),
                        RegisterGuard::Ready(Register::from_u32(*preg).unwrap()),
                    );

                    interval.location = Some(Location::Register(*preg));
                } else {
                    overlapping.push(vreg);
                }
            }
        }

        for vreg in overlapping.iter() {
            let acquired_preg;

            if let Some(preg) = phys_regs.pop() {
                acquired_preg = preg;
            } else {
                // All physical registers busy, look for dead virtual registers before pushing
                // one onto the stack.

                let mut swap_vreg = None;
                for vreg in map.keys() {
                    let (_, end) = last_uses.iter().find(|(v, _)| v == vreg).unwrap();
                    if *end <= op_idx {
                        swap_vreg = Some(*vreg);
                        break;
                    }
                }

                if let Some(dead) = swap_vreg {
                    let location = map.remove(&dead).unwrap();
                    match location {
                        Location::Register(preg) => acquired_preg = preg,
                        Location::Stack(offset) => {}
                    }
                    acquired_preg = preg;
                } else {
                    // All busy virtual registers are to be used in the future.
                    // Therefor we must save one register to the stack (preferrably the one to be
                    // used last) that is not currently overlapping.

                    let (furthest_use_vreg, next_use) = map
                        .keys()
                        .map(|vreg| {
                            lifetimes
                                .get(vreg)
                                .unwrap()
                                .next_use_after(op_idx)
                                .map(|next_use| (*vreg, next_use))
                                .unwrap_or((*vreg, usize::MAX))
                        })
                        .max_by_key(|(_, nu)| *nu)
                        .unwrap();

                    let preg = map.remove(&furthest_use_vreg).unwrap();

                    let stack_interval = Interval {
                        range: op_idx..(op_idx + 1),
                        location: Some(Location::Stack(stack_ptr)),
                    };

                    lifetimes
                        .get_mut(&furthest_use_vreg)
                        .unwrap()
                        .insert_interval(stack_interval);

                    acquired_preg = preg;
                    stack_ptr += 8;
                }
            }

            map.insert(*vreg, acquired_preg);
            lifetimes
                .get_mut(vreg)
                .unwrap()
                .set_location(op_idx, Some(Location::Register(acquired_preg as u32)));
        }
    }

    // TODO
    // difference between dest/src registers:
    // dead src reg can only be reused for a dest reg
    // on the same instruction, not another src reg.

    println!("Lifetimes ({}):", lifetimes.len());
    crate::ir::lifetime::print_lifetimes(&lifetimes);

    RegMap::new()
}
