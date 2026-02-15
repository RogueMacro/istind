use std::path::Path;

use crate::synthesize::arch::MachineCode;

#[cfg(target_os = "macos")]
pub mod mac;

pub trait Executable: Default {
    fn with_binary_identifier(self, ident: String) -> Self;

    fn build(&self, code: MachineCode, out_path: impl AsRef<Path>);
}

#[derive(Default)]
pub struct DummyExecutable;

impl Executable for DummyExecutable {
    fn with_binary_identifier(self, _ident: String) -> Self {
        self
    }

    fn build(&self, _code: MachineCode, _out_path: impl AsRef<Path>) {}
}
