use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use rand::prelude::*;

use kmerge::merge2_uniq;

fn bench_input<T>(len: usize, seed: u64) -> Vec<T>
where rand::distributions::Standard: Distribution<T>
{
    let rng = SmallRng::seed_from_u64(seed);
    rng.sample_iter(rand::distributions::Standard).take(len).collect()
}

fn bench_merge(c: &mut Criterion) {
    let mut a = bench_input::<(u64, u64)>(100000, 42);
    a.sort_unstable();
    a.dedup();

    let mut b = bench_input::<(u64, u64)>(100000, 35);
    b.sort_unstable();
    b.dedup();

    c.bench_function("naive 100k", |bench| bench.iter_batched(
        || (a.clone(), b.clone()),
        |(a, b)| merge2_uniq::naive(a, b),
        BatchSize::SmallInput,
    ));
    c.bench_function("vec::IntoIter 100k", |bench| bench.iter_batched(
        || (a.clone(), b.clone()),
        |(a, b)| merge2_uniq::into_iter(a, b),
        BatchSize::SmallInput,
    ));
    c.bench_function("vec::IntoIter safer 100k", |bench| bench.iter_batched(
        || (a.clone(), b.clone()),
        |(a, b)| merge2_uniq::into_iter_safer(a, b),
        BatchSize::SmallInput,
    ));
    c.bench_function("raw ptr 10k", |bench| bench.iter_batched(
        || (a.clone(), b.clone()),
        |(a, b)| merge2_uniq::raw_ptr(a, b),
        BatchSize::SmallInput,
    ));
    c.bench_function("safe 10k", |bench| bench.iter_batched(
        || (a.clone(), b.clone()),
        |(a, b)| merge2_uniq::old_datafrog(a, b),
        BatchSize::SmallInput,
    ));
}

criterion_group!(benches, bench_merge);
criterion_main!(benches);

