#[macro_use]
extern crate anyhow;

mod generator;
mod parser;
mod scanner;

use crate::scanner::{Span, Tokens};
use specifications::package::PackageIndex;
use brane_bvm::bytecode::Function;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct CompilerOptions {}

impl CompilerOptions {
    ///
    ///
    ///
    pub fn new() -> Self {
        CompilerOptions {}
    }
}

#[derive(Clone, Debug)]
pub struct CompilerState {}

impl CompilerState {
    ///
    ///
    ///
    pub fn new() -> Self {
        CompilerState {}
    }
}

#[derive(Clone, Debug)]
pub struct Compiler {
    pub options: CompilerOptions,
    pub package_index: PackageIndex,
    pub state: CompilerState,
}

impl Compiler {
    ///
    ///
    ///
    pub fn new(
        options: CompilerOptions,
        package_index: PackageIndex,
    ) -> Self {
        Compiler {
            state: CompilerState::new(),
            options,
            package_index,
        }
    }

    ///
    ///
    ///
    pub fn compile<S: Into<String>>(
        &mut self,
        input: S,
    ) -> Result<Function> {
        let input = input.into();
        let input = Span::new(&input);

        match scanner::scan_tokens(input) {
            Ok((_, tokens)) => {
                let tokens = Tokens::new(&tokens);

                match parser::parse_ast(tokens) {
                    Ok((_, program)) => {
                        generator::compile(program)
                    }
                    e => {
                        bail!("error from parser: {:?}", e);
                    }
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                bail!("Error from scanner: {:?}", e);
            }
            _ => bail!("Unkown error from scanner"),
        }
    }
}
