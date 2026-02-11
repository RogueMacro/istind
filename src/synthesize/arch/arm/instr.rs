#![allow(clippy::unusual_byte_groupings)]

use ux::{i12, i26, u12};

use super::reg::Register;

pub trait Instruction: std::fmt::Debug {
    fn encode(&self) -> u32;
}

impl Instruction for u32 {
    fn encode(&self) -> u32 {
        *self
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ImmShift {
    L0 = 0,
    L16 = 1,
    L32 = 2,
    L48 = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum Input<I> {
    Reg(Register),
    Imm(I),
}

pub fn add_imm(src_reg: Register, imm: i12, dest_reg: Register) -> u32 {
    let shift12 = 0;
    let imm = to_u32(imm);
    let src = src_reg as u32;
    let dest = dest_reg as u32;

    (0b100100010 << 23) & (shift12 << 22) & (imm << 10) & (src << 5) & dest
}

pub fn add_reg(reg_a: Register, reg_b: Register, dest_reg: Register) -> u32 {
    let a = reg_a as u32;
    let b = reg_b as u32;
    let dest = dest_reg as u32;

    (0b10001011 << 24) | (a << 16) | (b << 5) | dest
}

#[derive(Debug, Clone, Copy)]
pub struct Add {
    a: Register,
    b: Input<i12>,
    dest: Register,
}

impl Instruction for Add {
    fn encode(&self) -> u32 {
        let a = self.a as u32;
        let dest = self.dest as u32;

        match self.b {
            Input::Reg(reg) => (0b10001011 << 24) | (a << 16) | ((reg as u32) << 5) | dest,
            Input::Imm(imm) => {
                let imm: i32 = imm.into();
                (0b1001000100 << 22) | ((imm as u32) << 10) | (a << 5) | dest
            }
        }
    }
}

/// Branch with link instruction.
///
/// Stores pc+4 in lr and jumps to address.
///
/// Encoding:
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  0  0  1  0  1  imm26
///
/// - imm26: pc relative offset (in instructions, not bytes) to jump to
#[derive(Debug, Clone, Copy)]
pub struct BranchLink {
    pub addr: i26,
}

impl Instruction for BranchLink {
    fn encode(&self) -> u32 {
        let addr = (to_u32(self.addr)) & 0b000000_11111111111111111111111111;

        0b100101_00000000000000000000000000 | addr
    }
}

/// LDR instruction.
///
/// Encoding (unsigned offset):
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  1  1  1  0  0  1  0  1  imm12                               Rn             Rt
///
/// - imm12: offset from base (stored as a multiple of 8)
/// - Rn: base pointer
/// - Rt: destination register
#[derive(Debug, Clone, Copy)]
pub struct Load {
    pub stack_offset: u12,
    pub dest: Register,
}

impl Instruction for Load {
    fn encode(&self) -> u32 {
        let offset: u32 = self.stack_offset.into();
        let offset = offset / 8;
        let base = Register::SP as u32;
        let dest = self.dest as u32;

        (0b1111100101 << 22) | (offset << 10) | (base << 5) | dest
    }
}

/// MOV instruction.
///
/// Copies the value in the source register to the destination register.
///
/// Encoding:
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  0  1  0  1  0  1  0  0  0  0  Rm             0  0  0  0  0  0  1  1  1  1  1  Rd
///
/// - Rm: source register
/// - Rd: destination register
#[derive(Debug, Clone, Copy)]
pub struct MovReg {
    pub src: Register,
    pub dest: Register,
}

impl Instruction for MovReg {
    fn encode(&self) -> u32 {
        let src = self.src as u32;
        let dest = self.dest as u32;

        (0b10101010000_00000_00000011111 << 5) | (src << 16) | dest
    }
}

/// MOVZ instruction.
///
/// Moves a 16-bit immediate value into destination register, zeroing all "non-affected" bits. Can
/// be combined with [MOVK](Movk) to move a 32 or 64-bit value.
///
/// Encoding:
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  0  1  0  0  1  0  1  hw    imm16                                           Rd
///
/// - hw: shift left (0/16/32/48 encoded as 0/1/2/3)
/// - imm16: 16-bit immediate value to (optionally shift) into destination register
/// - Rd: destination register
#[derive(Debug, Clone, Copy)]
pub struct Movz {
    pub shift: ImmShift,
    pub imm_value: u16,
    pub dest_reg: Register,
}

impl Instruction for Movz {
    fn encode(&self) -> u32 {
        (0b110100101 << 23)
            | ((self.shift as u32) << 21)
            | ((self.imm_value as u32) << 5)
            | self.dest_reg as u32
    }
}

/// Return from subroutine to offset stored in link register.
///
/// Encoding:
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  0  1  0  1  1  0  0  1  0  1  1  1  1  1  0  0  0  0  0  0  Rn             0  0  0  0  0
///
/// - Rn: register containing jump address (always set to X30/LR)
#[derive(Debug, Clone, Copy)]
pub struct Ret;

impl Instruction for Ret {
    fn encode(&self) -> u32 {
        // 0b11110 -> 30 -> Link register
        // This is equivalent to: mov pc, lr
        0b1101011001011111000000_11110_00000
    }
}

/// # STR instruction.
///
/// Calculates an address from a base pointer/stack pointer and an offset, and saves a register
/// value to that address.
///
/// ## Encoding (register):
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  1  1  1  0  0  0  0  0  1  Rm             1  1  1  0  1  0  Rn             Rt
///
/// - Rm: offset register
/// - Rn: base pointer
/// - Rt: source register
///
/// ## Encoding (immediate, unsigned offset):
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  1  1  1  0  0  1  0  0  imm12                               Rn             Rt
///
/// - imm12: offset (stored as a multiple of 8)
/// - Rn: base pointer
/// - Rt: source register
#[derive(Debug, Clone, Copy)]
pub struct Store {
    pub base: Register,
    pub offset: Input<u12>,
    pub value: Register,
}

impl Instruction for Store {
    fn encode(&self) -> u32 {
        let base = self.base as u32;
        let value = self.value as u32;

        match self.offset {
            Input::Reg(reg) => {
                (0b11111000001_00000_111010 << 10) | ((reg as u32) << 16) | (base << 5) | value
            }
            Input::Imm(imm) => {
                let imm: u32 = imm.into();
                let imm = imm / 8;
                (0b1111100100 << 22) | (imm << 10) | (base << 5) | value
            }
        }
    }
}

/// SVC instruction.
///
/// Supervisor call. 0x80 counts as a valid immediate value. Call number should be stored in X16.
///
/// Encoding:
/// 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9  8  7  6  5  4  3  2  1  0
/// 1  1  0  1  0  1  0  0  0  0  0  imm16                                           0  0  0  0  1
///
/// - imm16: 16-bit immediate value
#[derive(Debug, Clone, Copy)]
pub struct Svc {
    imm: u16,
}

impl Instruction for Svc {
    fn encode(&self) -> u32 {
        0b11010100000_0000000000000000_00001 | ((self.imm as u32) << 5)
    }
}

/// Alias for [SVC](Svc).
///
/// Uses (by convention) 0x80 for the svc immediate. Syscall number is stored in X16.
#[derive(Debug, Clone, Copy)]
pub struct Syscall;

impl Instruction for Syscall {
    fn encode(&self) -> u32 {
        Svc { imm: 0x80 }.encode()
    }
}

fn to_u32(n: impl Into<i32>) -> u32 {
    n.into() as u32
}
