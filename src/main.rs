use std::{collections::HashMap, default, path::PathBuf};

use claks::{
    Compiler,
    ir::lifetime::{self, Interval, Lifetime},
};
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

    let compiler = Compiler::new();
    if compiler.compile("main.lk", "main").is_err() {
        return;
    }

    println!("Executable built and written to ./main");

    let exit_code = std::process::Command::new("otool")
        .arg("-vt")
        .arg("main")
        .status()
        .expect("program failed");
    // println!("{}", exit_code);

    // use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
    //
    // let mut colors = ColorGenerator::new();
    //
    // // Generate & choose some colours for each of our elements
    // let a = colors.next();
    // let b = colors.next();
    // let out = Color::Fixed(81);
    //
    // Report::build(ReportKind::Error, ("sample.tao", 12..12))
    //     .with_code(3)
    //     .with_message(format!("Incompatible types"))
    //     .with_label(
    //         Label::new(("sample.tao", 32..33))
    //             .with_message(format!("This is of type {}", "Nat".fg(a)))
    //             .with_color(a),
    //     )
    //     .with_label(
    //         Label::new(("sample.tao", 42..45))
    //             .with_message(format!("This is of type {}", "Str".fg(b)))
    //             .with_color(b),
    //     )
    //     .with_label(
    //         Label::new(("sample.tao", 11..48))
    //             .with_message(format!(
    //                 "The values are outputs of this {} expression",
    //                 "match".fg(out),
    //             ))
    //             .with_color(out),
    //     )
    //     .with_note(format!(
    //         "Outputs of {} expressions must coerce to the same type",
    //         "match".fg(out)
    //     ))
    //     .finish()
    //     .print(("sample.tao", Source::from(include_str!("sample.tao"))))
    //     .unwrap();
}
