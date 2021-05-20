use brane_bvm::bytecode;
use brane_bvm::vm::Vm;
use brane_dsl::{Compiler, CompilerOptions};
use specifications::package::PackageIndex;


const SIMPLE: &str = r#"
    func test(n) {
        if (n <= 1) {
            return 1;
        } else {
            return test(n - 1);
        }
    }

    let a := test(5);
    print(a);
"#;

fn compile() -> bytecode::Function {
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

    vm.main(function);
}
