use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use anyhow::Result;
use brane_bvm::{VM, VmCall, VmExecutor, VmOptions, bytecode::Function, values::Value};
use criterion::async_executor::FuturesExecutor;
use brane_dsl::{Compiler, CompilerOptions};
use specifications::package::PackageIndex;
use async_trait::async_trait;

const FIB_CODE: &str = r#"
    func fib(n) {
        if (n <= 1) {
            return 1;
        }

        return fib(n - 1) + fib(n - 2);
    }

    return fib(10);
"#;

#[derive(Clone)]
struct NoOpExecutor { }

#[async_trait]
impl VmExecutor for NoOpExecutor {
    async fn execute(&self, _call: VmCall) -> Result<Value> {
        Ok(Value::Unit)
    }
}

fn compile() -> Function {
    let mut compiler = Compiler::new(CompilerOptions::new(brane_dsl::Lang::BraneScript), PackageIndex::empty());
    compiler.compile(FIB_CODE).unwrap()
}

async fn do_something(f: Function) {
    let options = VmOptions { always_return: true };
    let executor = NoOpExecutor { };
    let mut vm = VM::new("bench", PackageIndex::empty(), None, Some(options), executor);

    vm.run(Some(f)).await;
}

fn from_elem(c: &mut Criterion) {
    c.bench_function("fib 10", move |b| {
        b.to_async(FuturesExecutor).iter(|| do_something(compile()));
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
