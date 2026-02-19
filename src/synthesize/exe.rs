use std::{path::Path, process::ExitStatus};

use crate::synthesize::arch::MachineCode;

#[cfg(target_os = "macos")]
pub mod mac;

pub trait Executable: Default {
    fn with_binary_identifier(self, ident: impl AsRef<str>) -> Self;

    fn build(&mut self, code: MachineCode, out_path: impl AsRef<Path>);

    fn run(&self) -> Result<ExitStatus, ExecutableError>;
}

#[derive(Default)]
pub struct DummyExecutable;

impl Executable for DummyExecutable {
    fn with_binary_identifier(self, _ident: impl AsRef<str>) -> Self {
        self
    }

    fn build(&mut self, _code: MachineCode, _out_path: impl AsRef<Path>) {}

    fn run(&self) -> Result<ExitStatus, ExecutableError> {
        Err(ExecutableError::Dummy)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutableError {
    #[error("executable was not built before running")]
    NoBuildPath,
    #[error("failed to run executable")]
    Io(#[from] std::io::Error),
    #[error("you cannot run a dummy executable")]
    Dummy,
}
