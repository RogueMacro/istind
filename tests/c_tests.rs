use std::{fs, path::Path, rc::Rc};

use claks::{
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

    let mut exe = AppleExecutable::default().with_binary_identifier("claks.test");
    exe.build(code, Path::new("ctarget/test").join(test_name));
    let status = exe.run().unwrap();

    assert_eq!(status.code(), Some(expect_exit_code));
}

#[test]
fn c_minimal_implicit() {
    compiles("int main() {}");
}

#[test]
fn c_minimal_return() {
    runs(
        "c_minimal_return",
        0,
        "
        int main() {
            return 0;
        }
        ",
    );
}

#[test]
fn c_minimal_return1() {
    runs(
        "c_minimal_return1",
        1,
        "
        int main() {
            return 1;
        }
        ",
    );
}

#[test]
fn c_variable() {
    runs(
        "c_variable",
        2,
        "
        int main() {
            int a = 2;
            return a;
        }
        ",
    );
}

#[test]
fn c_addition() {
    runs(
        "c_addition",
        5,
        "
        int main() {
            int a = 2;
            int b = 3;
            return a + b;
        }
        ",
    );
}

#[test]
fn c_function_call() {
    runs(
        "c_function_call",
        5,
        "
        int add(int a, int b) {
            return a + b;
        }

        int main() {
            return add(2, 3);
        }
        ",
    );
}
