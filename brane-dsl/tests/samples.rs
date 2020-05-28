use brane_dsl::compiler::{Compiler, CompilerOptions};
use brane_dsl::indexes::PackageIndex;
use std::fs;
use std::path::PathBuf;

#[test]
fn hello_world() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::none();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/hello-world.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
}
