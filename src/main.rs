//! Author: Rafael Bayer
//! Date: 2022-10-26
//!
//! Program: Brainfuck

use std::{fs, path::PathBuf};

use argh::FromArgs;
use brainfrick_rs::{compiler::compile, vm::VM};

#[derive(FromArgs)]
/// Brainfuck interpreter arguments.
/// Usage: `bfrs <filepath>`
struct Args {
    #[argh(positional, description = "brainfuck source file")]
    file: PathBuf,

    #[argh(switch, short = 's', description = "show compiled instructions")]
    show: bool,
}

fn main() {
    let args: Args = argh::from_env();
    let src = fs::read_to_string(args.file).expect("could not open file");
    let program = compile(&src);

    if args.show {
        println!("{program}");
    }

    let vm = VM::new(program);
    vm.run();
}
