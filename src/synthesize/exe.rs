use std::path::Path;

use crate::synthesize::arch::MachineCode;

#[cfg(target_os = "macos")]
pub mod mac;

pub trait Executable: Default {
    fn with_binary_identifier(self, ident: String) -> Self;

    fn build(&mut self, code: MachineCode, out_path: impl AsRef<Path>);

    fn run(&self) -> Result<(), ExecutableError>;
}

#[derive(Default)]
pub struct DummyExecutable;

impl Executable for DummyExecutable {
    fn with_binary_identifier(self, _ident: String) -> Self {
        self
    }

    fn build(&mut self, _code: MachineCode, _out_path: impl AsRef<Path>) {}

    fn run(&self) -> Result<(), ExecutableError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutableError {
    #[error("executable was not built before running")]
    NoBuildPath,
    #[error("failed to run executable")]
    Io(#[from] std::io::Error),
}
