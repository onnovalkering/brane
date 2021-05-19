use brane_bvm::{objects::Function, vm::Vm};
use brane_bvm::bytecode::Function as BFunction;
use brane_dsl::{Compiler, CompilerOptions};
use specifications::package::PackageIndex;


const SIMPLE: &str = r#"
    return 1 + 1;
"#;

fn compile() -> Function {
    let mut compiler = Compiler::new(
        CompilerOptions::new(brane_dsl::Lang::BraneScript),
        PackageIndex::empty(),
    );

    if let BFunction::UserDefined { chunk, name, arity } = compiler.compile(SIMPLE).unwrap() {
        dbg!(&chunk);

        return Function {
            arity,
            name,
            chunk,
        };
    }

    unreachable!()
}

fn main() {
    let function = compile();
    let mut vm = Vm::default();

    vm.main(function);
}
