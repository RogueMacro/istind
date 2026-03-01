use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

use clap::Parser;
use colored::Colorize;
use istind::{Compiler, synthesize::exe::mac::AppleExecutable};

#[derive(Parser)]
#[command(version)]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    if let Err(err) = run(args) {
        eprintln!("{} {}", "error:".bright_red().bold(), err);
    }

    Ok(())
}

fn run(args: Cli) -> Result<(), Error> {
    let Some(module) = args.file.file_stem() else {
        return Err(Error::InvalidFile);
    };

    let compiler = Compiler::<AppleExecutable>::default();

    let start = Instant::now();
    if let Err(error_count) = compiler.compile(&args.file, target_mod(module)?) {
        return Err(Error::CompilationFailed(error_count));
    }
    let end = Instant::now();
    let dur = end - start;

    println!(
        "{:>12} {} in {:.2}s",
        "Compiled".bright_green().bold(),
        module.to_string_lossy(),
        dur.as_secs_f32(),
    );

    std::process::Command::new("otool")
        .arg("-vt")
        .arg("btarget/main")
        .status()
        .expect("program failed");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("path is not a compilable file")]
    InvalidFile,
    #[error("failed to compile due to {0} errors")]
    CompilationFailed(usize),
    #[error("io error")]
    Io(#[from] io::Error),
}

fn target_mod(module: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let target_dir = Path::new("btarget");
    if !target_dir.exists() {
        fs::create_dir(target_dir)?;
    }

    Ok(target_dir.join(module.as_ref()))
}
