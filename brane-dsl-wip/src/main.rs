use anyhow::Result;
use nom::error::VerboseError;
use brane_dsl_wip::{
    compiler::{compile},
    parser,
    scanner::{self, Tokens},
    vm::VM,
};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;

fn main() -> Result<()> {
    let input =
        Span::new("class Brioche {} print(Brioche);");

    match scanner::scan_tokens::<VerboseError<Span>>(input) {
        Ok((_, tokens)) => {
            let tokens = Tokens::new(&tokens);
            match parser::parse_program::<VerboseError<Tokens>>(tokens) {
                Ok((_, program)) => {
                    dbg!(&program);
                    let main_func = compile(program)?;

                    let mut vm = VM::new(main_func);
                    vm.run();
                }
                Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                    println!("error from parser: {:?}", e);
                }
                a => {
                    dbg!(&a);
                }
            }
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            println!("error from scanner: {:?}", e);
        }
        _ => {}
    }

    Ok(())
}
