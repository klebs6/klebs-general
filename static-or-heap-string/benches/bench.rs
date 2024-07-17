extern crate criterion;
extern crate static_or_heap_string;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use static_or_heap_string::StaticOrHeapString;

fn bench_static_or_heap_string(c: &mut Criterion) {
    let static_str = StaticOrHeapString::Static("hello");
    let heap_str = StaticOrHeapString::Heap(String::from("world"));

    c.bench_function("StaticOrHeapString::as_str (static)", |b| {
        b.iter(|| black_box(static_str.as_str()))
    });

    c.bench_function("StaticOrHeapString::as_str (heap)", |b| {
        b.iter(|| black_box(heap_str.as_str()))
    });

    c.bench_function("StaticOrHeapString::is_empty (static)", |b| {
        b.iter(|| black_box(static_str.is_empty()))
    });

    c.bench_function("StaticOrHeapString::is_empty (heap)", |b| {
        b.iter(|| black_box(heap_str.is_empty()))
    });

    c.bench_function("StaticOrHeapString::len (static)", |b| {
        b.iter(|| black_box(static_str.len()))
    });

    c.bench_function("StaticOrHeapString::len (heap)", |b| {
        b.iter(|| black_box(heap_str.len()))
    });
}

criterion_group!(benches, bench_static_or_heap_string);
criterion_main!(benches);

