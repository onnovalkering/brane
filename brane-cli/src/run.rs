use crate::{registry};
use anyhow::Result;
use brane_bvm::{VmOptions};
use brane_bvm::{VmResult, VM};
use brane_dsl::{Compiler, CompilerOptions, Lang};
use std::fs;
use std::path::PathBuf;

///
///
///
pub async fn handle(
    file: PathBuf,
) -> Result<()> {
    let source_code = fs::read_to_string(&file)?;

    let compiler_options = CompilerOptions::new(Lang::BraneScript);
    let package_index = registry::get_package_index().await?;
    let mut compiler = Compiler::new(compiler_options, package_index.clone());

    let options = VmOptions { always_return: true };
    let mut vm = VM::new("local-run", package_index, None, Some(options));

    match compiler.compile(source_code) {
        Ok(function) => {
            vm.call(function, 0);

            loop {
                match vm.run(None) {
                    VmResult::Call(_) => {
                        todo!();
                    }
                    VmResult::Ok(value) => {
                        let output = value.map(|v| format!("{:?}", v)).unwrap_or_default();
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                        break;
                    }
                    VmResult::RuntimeError => {
                        eprintln!("Runtime error!");
                        break;
                    }
                }
            }
        },
        Err(error) => eprintln!("{:?}", error)
    }

    Ok(())
}
