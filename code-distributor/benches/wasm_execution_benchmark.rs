use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::Value;

use code_distributor::fragment_executor::wasmer_runtime::WasmerInstance;

const FRAGMENT_PATH: &str = "benches/resources/wasm";

fn fibonacci_benchmark(c: &mut Criterion) {
    let parameters = vec![20, 40, 50];
    let function_name = "execute__fibonacci";
    let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");

    let mut group = c.benchmark_group("Fibonacci");
    for param in parameters {
        group.sample_size(10);
        group.bench_with_input(BenchmarkId::from_parameter(param), &param, |b, &param| {
            let params: Vec<Value> = vec![Value::from(param)];
            b.iter(|| {
                WasmerInstance::new(fragment_path.clone(), function_name.to_string())
                    .execute(black_box(&params))
            });
        });
    }
    group.finish();
}

fn factorial_benchmark(c: &mut Criterion) {
    let parameters = vec![12, 16, 20];
    let function_name = "execute__factorial";
    let fragment_path = PathBuf::from(FRAGMENT_PATH).join("factorial.wasm");

    let mut group = c.benchmark_group("Factorial");
    for param in parameters {
        group.sample_size(10);
        group.bench_with_input(BenchmarkId::from_parameter(param), &param, |b, &param| {
            let params: Vec<Value> = vec![Value::from(param)];
            b.iter(|| {
                WasmerInstance::new(fragment_path.clone(), function_name.to_string())
                    .execute(black_box(&params))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, fibonacci_benchmark, factorial_benchmark);
criterion_main!(benches);
