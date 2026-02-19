use std::{marker::PhantomData, path::Path, rc::Rc};

use ariadne::Source;

use crate::{
    analyze::{Error, ast::parse::Parser, lex::Lexer},
    ir::IR,
    synthesize::{
        arch::{Assemble, MachineCode, arm::ArmAssembler},
        exe::Executable,
    },
};

pub mod analyze;
pub mod ir;
pub mod synthesize;

#[derive(Default)]
pub struct Compiler<E: Executable> {
    _marker: PhantomData<E>,
}

impl<E: Executable> Compiler<E> {
    pub fn compile(&self, path: impl AsRef<Path>, out_path: impl AsRef<Path>) -> Result<(), ()> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path).unwrap();

        let source_name = Rc::new(
            path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or(String::from("unknown")),
        );

        let code = match self.compile_source(source_name.clone(), &source) {
            Ok(code) => code,
            Err(e) => {
                e.eprint((source_name, Source::from(source))).unwrap();
                return Err(());
            }
        };

        E::default()
            .with_binary_identifier(source_name.as_ref())
            .build(code, out_path);

        Ok(())
    }

    pub fn compile_source(&self, name: Rc<String>, source: &str) -> Result<MachineCode, Error> {
        let lexer = Lexer::new(name.clone(), source)?;
        let parser = Parser::new(name, lexer);
        let ast = parser.into_ast()?;

        let ir = IR::generate(ast);

        let code = ArmAssembler::assemble(ir);

        Ok(code)
    }
}
