use brainfuck_rs::{interpreter::Interpreter, io::NoIO};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn bench_hello_world(c: &mut Criterion) {
    let src = include_str!("..\\samples\\helloworld.bf");

    c.bench_function("hello world", |b| {
        b.iter(|| Interpreter::new_with_io(black_box(src), black_box(NoIO {})).run());
    });
}

criterion_group!(benches, bench_hello_world);
criterion_main!(benches);
