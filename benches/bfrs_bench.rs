use std::fs;

use brainfuck_rs::{compiler::compile, io::NoIO, vm::VM};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn bench_hello_world(c: &mut Criterion) {
    let src = fs::read_to_string("samples\\helloworld.bf").unwrap();
    bench_program(c, "Hello World", &src)
}

pub fn bench_pi_4(c: &mut Criterion) {
    let src = fs::read_to_string("samples\\pi4.bf").unwrap();
    bench_program(c, "4 digits of pi", &src)
}

pub fn bench_sierpinski(c: &mut Criterion) {
    let src = fs::read_to_string("samples\\sierpinski.bf").unwrap();
    bench_program(c, "sierpinski's triangle", &src);
}

pub fn bench_fib11(c: &mut Criterion) {
    let src = fs::read_to_string("samples\\fib11.bf").unwrap();
    bench_program(c, "Fib 11", &src);
}

fn bench_program(c: &mut Criterion, name: &str, src: &str) {
    let prog = compile(src);
    let vm = VM::new_with_io(prog, NoIO {});
    c.bench_function(name, |b| {
        b.iter_batched(|| vm.clone(), |mut vm| vm.run(), criterion::BatchSize::SmallInput)
    });
}

criterion_group!(
    benches,
    bench_hello_world,
    bench_pi_4,
    bench_sierpinski,
    bench_fib11
);
criterion_main!(benches);
