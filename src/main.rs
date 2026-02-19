use std::{
    fs, io,
    path::{Path, PathBuf},
};

use claks::{Compiler, synthesize::exe::mac::AppleExecutable};
use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
#[command(version)]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    let Some(module) = args.file.file_stem() else {
        return Err(Error::InvalidFile);
    };

    let compiler = Compiler::<AppleExecutable>::default();
    if compiler.compile(&args.file, target_mod(module)?).is_err() {
        return Err(Error::CompilationFailed);
    }

    println!(
        "{:>12} module {}",
        "Compiled".bright_green(),
        module.to_string_lossy()
    );

    let exit_code = std::process::Command::new("otool")
        .arg("-vt")
        .arg("main")
        .status()
        .expect("program failed");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("path is not a compilable file")]
    InvalidFile,
    #[error("compilation failed")]
    CompilationFailed,
    #[error("io error")]
    Io(#[from] io::Error),
}

fn target_mod(module: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let target_dir = Path::new("ctarget");
    if !target_dir.exists() {
        fs::create_dir(target_dir)?;
    }

    Ok(target_dir.join(module.as_ref()))
}
