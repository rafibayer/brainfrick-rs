//! Author: Rafael Bayer
//! Date: 2022-10-26
//!
//! Program: Brainfuck

use std::{fs, path::PathBuf};

use argh::FromArgs;
use brainfuck_rs::{compiler::compile, vm::VM};

#[derive(FromArgs)]
/// Brainfuck interpreter arguments.
/// Usage: `bfrs <filepath>`
struct Args {
    #[argh(positional, description = "brainfuck source file")]
    file: PathBuf,
}

fn main() {
    let args: Args = argh::from_env();
    let src = fs::read_to_string(args.file).expect("could not open file");
    // let src = fs::read_to_string("samples\\fib3.bf").expect("could not open file");

    let program = compile(&src);
    // println!("{}", &program);

    let mut vm = VM::new(program);
    vm.run();

    // println!("\n{}", vm);
}
