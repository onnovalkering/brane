use crate::{docker::DockerExecutor, registry};
use anyhow::Result;
use brane_bvm::vm::Vm;
use brane_dsl::{Compiler, CompilerOptions, Lang};
use std::fs;
use std::path::PathBuf;

///
///
///
pub async fn handle(
    file: PathBuf,
    data: Option<PathBuf>,
) -> Result<()> {
    let source_code = fs::read_to_string(&file)?;

    let compiler_options = CompilerOptions::new(Lang::BraneScript);
    let package_index = registry::get_package_index().await?;
    let mut compiler = Compiler::new(compiler_options, package_index.clone());

    let executor = DockerExecutor::new(data);
    let mut vm = Vm::new_with(executor, Some(package_index), None);

    match compiler.compile(source_code) {
        Ok(function) => vm.main(function).await,
        Err(error) => eprintln!("{:?}", error),
    }

    Ok(())
}
