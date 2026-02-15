use crate::ir::IR;

pub mod arm;

#[derive(Default)]
pub struct MachineCode {
    pub instructions: Vec<u8>,
    pub entry_point_offset: u64,
}

impl MachineCode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            entry_point_offset: 0,
        }
    }
}

pub trait Assemble {
    fn assemble(ir: IR) -> MachineCode;
}
