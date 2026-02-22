use std::{fs, path::Path, rc::Rc};

use istind::{
    Compiler,
    synthesize::exe::{DummyExecutable, Executable, mac::AppleExecutable},
};

fn mod_main() -> Rc<String> {
    Rc::new(String::from("main"))
}

fn compiles(source: &str) {
    let compiler: Compiler<DummyExecutable> = Compiler::default();
    compiler.compile_source(mod_main(), source).unwrap();
}

fn runs(test_name: &str, expect_exit_code: i32, source: &str) {
    let compiler: Compiler<DummyExecutable> = Compiler::default();
    let code = compiler.compile_source(mod_main(), source).unwrap();

    fs::create_dir_all("ctarget/test").unwrap();

    let mut exe = AppleExecutable::default().with_binary_identifier("istind.test");
    exe.build(code, Path::new("ctarget/test").join(test_name));
    let status = exe.run().unwrap();

    assert_eq!(status.code(), Some(expect_exit_code));
}

#[test]
fn minimal_implicit() {
    compiles("fn main() {}");
}

#[test]
fn minimal_return() {
    runs(
        "minimal_return",
        0,
        "
        fn main() {
            return 0;
        }
        ",
    );
}

#[test]
fn minimal_return1() {
    runs(
        "minimal_return1",
        1,
        "
        fn main() {
            return 1;
        }
        ",
    );
}

#[test]
fn assignment() {
    runs(
        "assignment",
        2,
        "
        fn main() {
            let a = 2;
            return a;
        }
        ",
    );
}

#[test]
fn addition() {
    runs(
        "addition",
        5,
        "
        fn main() {
            let a = 2;
            let b = 3;
            return a + b;
        }
        ",
    );
}
