use brane_dsl::compiler::{Compiler, CompilerOptions};
use brane_dsl::indexes::PackageIndex;
use std::fs;
use std::path::PathBuf;

#[test]
fn hello_world() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/hello-world.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    println!("{:#?}", instructions);

    assert!(instructions.len() > 0);
}

#[test]
fn wait_until() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/wait-until.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
}

#[test]
fn while_loop() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/while-loop.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
}

#[test]
fn if_else() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/if-else.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
    println!("{:#?}", instructions);
}

#[test]
fn files() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/files.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
    println!("{:#?}", instructions);
}

#[test]
fn imports() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/imports.bk").unwrap();
    let _ = compiler.compile(&program).unwrap();

    assert!(compiler.state.imports.len() > 0);
}

#[test]
fn defaults() {
    let index = PackageIndex::from_path(&PathBuf::from("./resources/packages.json")).unwrap();

    let options = CompilerOptions::default();
    let mut compiler = Compiler::new(options, index).unwrap();

    let program = fs::read_to_string("./resources/defaults.bk").unwrap();
    let instructions = compiler.compile(&program).unwrap();

    assert!(instructions.len() > 0);
    println!("{:#?}", instructions);
}
