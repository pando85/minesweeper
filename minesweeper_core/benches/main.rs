use minesweeper_core::get_grid;

use criterion::{black_box, criterion_group, criterion_main, Criterion};


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("get_grid 1000 10000 50000", |b| b.iter(|| get_grid(black_box(1000), black_box(10000), black_box(90000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
