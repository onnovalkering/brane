use brane_bvm::bytecode;
use brane_bvm::vm::Vm;
use brane_dsl::{Compiler, CompilerOptions};
use specifications::package::PackageIndex;

const SIMPLE: &str = r#"
    func test(n) {
        print(1 + n);
        return 2;
    }

    let a := test(5);
"#;

fn compile() -> bytecode::FunctionMut {
    let mut compiler = Compiler::new(
        CompilerOptions::new(brane_dsl::Lang::BraneScript),
        PackageIndex::empty(),
    );

    compiler.compile(SIMPLE).unwrap()
}

fn main() {
    let function = compile();

    dbg!(&function.chunk);
    println!();

    let mut vm = Vm::default();

    futures::executor::block_on(vm.main(function));
}
