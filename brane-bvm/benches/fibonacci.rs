use brane_bvm::{bytecode::FunctionMut, executor::NoOpExecutor, vm::Vm};
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

fn compile(n: u8) -> FunctionMut {
    let mut compiler = Compiler::new(
        CompilerOptions::new(brane_dsl::Lang::BraneScript),
        PackageIndex::empty(),
    );

    compiler.compile(FIB_CODE.replace("xyz", &format!("{}", n))).unwrap()
}

async fn run(f: FunctionMut) {
    let mut vm = Vm::<NoOpExecutor>::default();
    vm.main(f).await;
}

fn from_elem(c: &mut Criterion) {
    c.bench_function("fib 5", |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(5)));
    });
    c.bench_function("fib 10", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(10)));
    });
    c.bench_function("fib 15", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(15)));
    });
    c.bench_function("fib 20", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(20)));
    });
    c.bench_function("fib 25", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(25)));
    });
    c.bench_function("fib 30", move |b| {
        b.to_async(FuturesExecutor).iter(|| run(compile(30)));
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
