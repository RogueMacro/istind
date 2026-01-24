use std::{fs, path::Path, process};

pub fn assemble(file: &Path) {
    fs::create_dir_all("mylang_target").unwrap();

    let obj_file = Path::new("mylang_target").join(file).with_extension("o");
    process::Command::new("as")
        .args(file)
        .arg("-o")
        .arg(&obj_file)
        .arg("-g")
        .status()
        .unwrap();

    link(&obj_file);
}

fn link(file: &Path) {
    let program_file = file.with_extension("");

    process::Command::new("ld")
        .arg(file)
        .arg("-o")
        .arg(program_file)
        .arg("-lSystem")
        .arg("-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib")
        .status()
        .unwrap();
}
