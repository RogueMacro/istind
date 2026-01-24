use std::{path::PathBuf, process};

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(long)]
    asm: bool,
    file: PathBuf,
}

fn main() {
    let args = Cli::parse();

    assert!(args.asm, "Only assembly supported");

    mylang::asm::assemble(&args.file);

    if let Err(code) = process::Command::new("mylang_target/main").status() {
        println!("exit code: {}", code);
    }
}
