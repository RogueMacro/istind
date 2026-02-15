use std::path::PathBuf;

use claks::{Compiler, synthesize::exe::DummyExecutable};
use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(long)]
    asm: bool,
    file: PathBuf,
}

fn main() {
    // let args = Cli::parse();

    let compiler = Compiler::<DummyExecutable>::default();
    if compiler.compile("main.lk", "main").is_err() {
        return;
    }

    // println!("Executable built and written to ./main");

    // let exit_code = std::process::Command::new("otool")
    //     .arg("-vt")
    //     .arg("main")
    //     .status()
    //     .expect("program failed");
}
