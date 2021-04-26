#[macro_use]
extern crate anyhow;

#[path = "generator/generator.rs"]
mod generator;

#[path = "parser/parser.rs"]
mod parser;

#[path = "scanner/scanner.rs"]
mod scanner;

use crate::scanner::{Span, Tokens};
use anyhow::Result;
use brane_bvm::bytecode::Function;
use specifications::package::PackageIndex;
use nom::error::{VerboseError, VerboseErrorKind};

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
                    Ok((_, program)) => generator::compile(program),
                    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        bail!("{}", convert_parser_error(tokens, e));
                    },
                    _ => bail!("unkown error from parser"),
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                bail!("{}", convert_scanner_error(input, e));
            }
            _ => bail!("Unkown error from scanner"),
        }
    }
}

pub fn convert_parser_error(
    input: Tokens,
    e: VerboseError<Tokens>,
) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, (tokens, kind)) in e.errors.iter().enumerate() {
        match kind {
            VerboseErrorKind::Char(c) => {
                if tokens.tok.is_empty() {
                    if let Some(mismatch) = input.tok.last() {
                        let mismatch = mismatch.inner();
                        let line = String::from_utf8(mismatch.get_line_beginning().to_vec()).unwrap();
                        let line_number = mismatch.location_line();
                        let column_number = mismatch.get_column() + 1;

                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\n\
                             {line}\n\
                             {caret:>column$}\n\
                             expected '{expected}', but encountered EOF\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                          ).unwrap();
                    } else {
                        write!(
                            &mut result,
                            "{i}: expected '{expected}', but EOF\n\n",
                            i = i,
                            expected = c,
                          ).unwrap();
                    }
                } else {
                    let mismatch = tokens.tok[0].inner();
                    let line = String::from_utf8(mismatch.get_line_beginning().to_vec()).unwrap();
                    let line_number = mismatch.location_line();
                    let column_number = mismatch.get_column();
                    let actual = mismatch.fragment();

                    write!(
                        &mut result,
                        "{i}: at line {line_number}:\n\n\
                         {line}\n\
                         {caret:>column$}\n\
                         expected '{expected}', found '{actual}'\n\n",
                        i = i,
                        line_number = line_number,
                        line = line,
                        caret = '^',
                        column = column_number,
                        expected = c,
                        actual = actual,
                      ).unwrap();
                }
            },
            VerboseErrorKind::Nom(nom::error::ErrorKind::Tag) => {
                dbg!(&tokens);

                let mismatch = tokens.tok[0].inner();
                let line = String::from_utf8(mismatch.get_line_beginning().to_vec()).unwrap();
                let line_number = mismatch.location_line();
                let column_number = mismatch.get_column();
                let actual = mismatch.fragment();

                write!(
                    &mut result,
                    "{i}: at line {line_number}:\n\
                     {line}\n\
                     {caret:>column$}\n\
                     unexpected token '{actual}'\n\n",
                    i = i,
                    line_number = line_number,
                    line = line,
                    caret = '^',
                    column = column_number,
                    actual = actual,
                  ).unwrap();
            },
            VerboseErrorKind::Context(s) => {
                let mismatch = tokens.tok[0].inner();
                let line = String::from_utf8(mismatch.get_line_beginning().to_vec()).unwrap();

                writeln!(result, "{} in section '{}', at: {}", i, s, line).unwrap()
            }
            e => { dbg!(&e); }
        }
    }

    result
}


pub fn convert_scanner_error(
    input: Span,
    e: VerboseError<Span>,
  ) -> String {
    use std::fmt::Write;
    use nom::Offset;

    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
      let offset = input.offset(substring);

      if input.is_empty() {
        match kind {
          VerboseErrorKind::Char(c) => {
            write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
          }
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

