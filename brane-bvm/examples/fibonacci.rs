use anyhow::Result;
use async_trait::async_trait;
use brane_bvm::{bytecode::Function, values::Value, VmCall, VmExecutor, VmOptions, VM};
use brane_dsl::{Compiler, CompilerOptions};
use specifications::package::PackageIndex;

const FIB_CODE: &str = r#"
    func fib(n) {
        if (n <= 1) {
            return 1;
        }

        return fib(n - 1) + fib(n - 2);
    }

    return fib(15);
"#;

#[derive(Clone)]
struct NoOpExecutor {}

#[async_trait]
impl VmExecutor for NoOpExecutor {
    async fn execute(
        &self,
        _call: VmCall,
    ) -> Result<Value> {
        Ok(Value::Unit)
    }
}

fn compile() -> Function {
    let mut compiler = Compiler::new(
        CompilerOptions::new(brane_dsl::Lang::BraneScript),
        PackageIndex::empty(),
    );
    compiler.compile(FIB_CODE).unwrap()
}

#[tokio::main]
async fn main() {
    let run = || {
        let f = compile();

        let options = VmOptions { always_return: true };
        let executor = NoOpExecutor {};
        let mut vm = VM::new("bench", PackageIndex::empty(), None, Some(options), executor);

        let value = futures::executor::block_on(vm.run(Some(f))).unwrap();
        println!("{:?}", value);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/", run).unwrap();
    }

}
