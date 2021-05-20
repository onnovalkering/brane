use brane_bvm::{bytecode::Function, vm::Vm};
use brane_dsl::{Compiler, CompilerOptions};
use criterion::async_executor::FuturesExecutor;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use specifications::package::PackageIndex;

const FIB_CODE: &str = r#"
    func fib(n) {
        if (n <= 1) {
            return 1;
        }

        return fib(n - 1) + fib(n - 2);
    }

    fib(xyz);
"#;

#[derive(Clone)]
struct NoOpExecutor {}

fn compile(n: u8) -> Function {
    let mut compiler = Compiler::new(
        CompilerOptions::new(brane_dsl::Lang::BraneScript),
        PackageIndex::empty(),
    );

    compiler.compile(FIB_CODE.replace("xyz", &format!("{}", n))).unwrap()
}

async fn run(f: Function) {
    let mut vm = Vm::default();
    vm.main(f);
}

fn from_elem(c: &mut Criterion) {
    c.bench_function("fib 5", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(5)));
    });
    c.bench_function("fib 10", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(10)));
    });
    c.bench_function("fib 15", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(15)));
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
