#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use crate::parser::ast::Program;
use crate::scanner::Tokens;
use anyhow::Result;
use nom::error::{VerboseError, VerboseErrorKind};

pub mod compiler;
pub mod parser;
pub mod scanner;
pub mod vm;

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;

pub fn scan_and_parse<S: Into<String>>(source_code: S) -> Result<Program> {
    let source_code = source_code.into();
    let input = Span::new(&source_code);

    match scanner::scan_tokens::<VerboseError<Span>>(input) {
        Ok((_, tokens)) => {
            let tokens = Tokens::new(&tokens);
            match parser::parse_program::<VerboseError<Tokens>>(tokens) {
                Ok((_, program)) => {
                    return Ok(program);
                }
                e => {
                    bail!("error from parser: {:?}", e);
                }
            }
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            eprintln!("{}", convert_error(input, e));
            bail!("Error from scanner");
        }
        _ => bail!("Unkown error from scanner"),
    }
}

///
pub fn convert_error(
    input: Span,
    e: VerboseError<Span>,
) -> String {
    use nom::Offset;
    use std::fmt::Write;

    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let offset = input.offset(substring);

        if input.is_empty() {
            match kind {
                VerboseErrorKind::Char(c) => write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c),
                VerboseErrorKind::Context(s) => write!(&mut result, "{}: in {}, got empty input\n\n", i, s),
                VerboseErrorKind::Nom(e) => write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e),
            }
        } else {
            let prefix = &input.as_bytes()[..offset];

            // Count the number of newlines in the first `offset` bytes of input
            let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

            // Find the line that includes the subslice:
            // Find the *last* newline before the substring starts
            let line_begin = prefix
                .iter()
                .rev()
                .position(|&b| b == b'\n')
                .map(|pos| offset - pos)
                .unwrap_or(0);

            // Find the full line after that newline
            let line = input[line_begin..]
                .lines()
                .next()
                .unwrap_or(&input[line_begin..])
                .trim_end();

            // The (1-indexed) column number is the offset of our substring into that line
            let column_number = line.offset(substring) + 1;

            match kind {
                VerboseErrorKind::Char(c) => {
                    if let Some(actual) = substring.chars().next() {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                 {line}\n\
                 {caret:>column$}\n\
                 expected '{expected}', found {actual}\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                            actual = actual,
                        )
                    } else {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                 {line}\n\
                 {caret:>column$}\n\
                 expected '{expected}', got end of input\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                        )
                    }
                }
                VerboseErrorKind::Context(s) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {context}:\n\
               {line}\n\
               {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    context = s,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
                VerboseErrorKind::Nom(e) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {nom_err:?}:\n\
               {line}\n\
               {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    nom_err = e,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
            }
        }
        // Because `write!` to a `String` is infallible, this `unwrap` is fine.
        .unwrap();
    }

    result
}
