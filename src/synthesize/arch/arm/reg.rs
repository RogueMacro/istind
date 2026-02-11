use strum::EnumIter;

pub type Reg = Register;

#[repr(u32)]
#[derive(EnumIter, Clone, Copy, Debug, PartialEq)]
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

pub struct RegAlloc {
    scratch: [RegStatus; 16],   // X0-X15
    preserved: [RegStatus; 10], // X19-X28
}

impl RegAlloc {
    pub fn new() -> Self {
        let scratch = [RegStatus::Available; _];
        let preserved = [RegStatus::Available; _];

        Self { scratch, preserved }
    }

    // pub fn get_any(&mut self, asm: &mut Assembler) {}

    pub fn get_reg(&mut self, reg: Register) {}
}

impl Default for RegAlloc {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum RegStatus {
    Available,
    Used,
}
