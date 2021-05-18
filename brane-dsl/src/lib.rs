#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod errors;
#[path = "generator/generator.rs"]
mod generator;
#[path = "parser/parser.rs"]
mod parser;
#[path = "scanner/scanner.rs"]
mod scanner;

use crate::parser::{bakery, bscript};
use crate::scanner::{Span, Tokens};
use anyhow::Result;
use brane_bvm::bytecode::Function;
use specifications::package::PackageIndex;

#[derive(Clone, Debug)]
pub enum Lang {
    Bakery,
    BraneScript,
}

#[derive(Clone, Debug)]
pub struct CompilerOptions {
    pub lang: Lang,
}

impl CompilerOptions {
    ///
    ///
    ///
    pub fn new(lang: Lang) -> Self {
        CompilerOptions { lang }
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
                let program = match self.options.lang {
                    Lang::Bakery => bakery::parse_ast(tokens, self.package_index.clone()),
                    Lang::BraneScript => bscript::parse_ast(tokens),
                };

                match program {
                    Ok((_, program)) => generator::compile(program),
                    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        bail!("{}", errors::convert_parser_error(tokens, e));
                    }
                    _ => bail!("Compiler error: unkown error from parser."),
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                bail!("{}", errors::convert_scanner_error(input, e));
            }
            _ => bail!("Compiler error: Unkown error from scanner."),
        }
    }
}
