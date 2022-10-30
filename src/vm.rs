use std::{fmt::Display, str::from_utf8};

use crate::{
    compiler::Program,
    instruction::Instruction,
    io::{InputOutput, StdIO},
};

/// Default memory size for VM.
const MEM: usize = 30_000;

/// Brainfuck VM.
#[derive(Debug, Clone)]
pub struct VM<IO: InputOutput> {
    program: Program,

    /// Program Memory
    data: Box<[u8; MEM]>,

    /// Instruction Pointer
    in_ptr: usize,

    /// Memory pointer
    d_ptr: usize,

    /// InputOutput implementation
    io: IO,
}

impl<IO: InputOutput> Display for VM<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("{\n");
        buf.push_str(&format!("\tins: {}\n", self.in_ptr));
        buf.push_str(&format!("\tptr: {}\n", self.d_ptr));
        let mut last_nonzero = 0;
        for i in 0..MEM {
            if self.data[i] != 0 {
                last_nonzero = i;
            }
        }

        let used_mem = &self.data[..=last_nonzero];
        buf.push_str("\tmem: [");
        for b in used_mem {
            buf.push_str(&format!("{}, ", b));
        }

        write!(f, "{}]\n}}", buf)
    }
}

impl VM<StdIO> {
    pub fn new(program: Program) -> Self {
        VM::new_with_io(program, StdIO {})
    }
}

impl<IO: InputOutput> VM<IO> {
    pub fn new_with_io(program: Program, io: IO) -> Self {
        VM {
            program,
            data: Box::new([0; MEM]),
            in_ptr: 0,
            d_ptr: 0,
            io,
        }
    }

    pub fn run(&mut self) {
        while self.in_ptr < self.program.instructions.len() {
            // unless we encounter a loop, we will go to the next
            // instruciton after this one
            let mut next_ins_ptr = self.in_ptr + 1;

            // instruction implementations
            let ins = self.program.instructions[self.in_ptr];
            match ins {
                Instruction::Shift(count) => self.d_ptr = ((self.d_ptr as isize + count) as usize) % MEM,
                Instruction::Alt(amount) => {
                    let new_value = match amount {
                        _ if amount > 0 => self.data[self.d_ptr].wrapping_add(amount as u8),
                        _ if amount < 0 => self.data[self.d_ptr].wrapping_sub(-amount as u8),
                        _ => unreachable!(),
                    };
                    self.data[self.d_ptr] = new_value;
                }
                Instruction::Out => self.io.print(self.data[self.d_ptr]),
                Instruction::In => self.data[self.d_ptr] = self.io.getch(),
                Instruction::Loop => {
                    if self.data[self.d_ptr] == 0u8 {
                        next_ins_ptr = self.program.loop_map[&self.in_ptr] + 1;
                    }
                }
                Instruction::End => {
                    if self.data[self.d_ptr] != 0u8 {
                        next_ins_ptr = self.program.loop_map[&self.in_ptr] + 1;
                    }
                }
                Instruction::Clear => {
                    // optimized version of [-]
                    self.data[self.d_ptr] = 0u8;
                }
                Instruction::Copy { mul, offset } => {
                    let target_d_ptr = ((self.d_ptr as isize + offset) as usize) % MEM;
                    let new_value =
                        self.data[target_d_ptr].wrapping_add(self.data[self.d_ptr].wrapping_mul(mul));
                    self.data[target_d_ptr] = new_value;
                }
                Instruction::NoOp => {
                    panic!("NoOp found in program!");
                }
            };

            self.in_ptr = next_ins_ptr;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, rc::Rc};

    use crate::{
        compiler::compile,
        io::{NoIO, TestIO},
    };

    use super::*;

    // todo: this test will break if compiler
    // change the braces in the output program
    #[test]
    fn test_bracket_matching() {
        let p = compile("[->.<][[]][]");
        let i = VM::new(p);

        assert_eq!(8, i.program.loop_map.len());
        assert_eq!(0, i.program.loop_map[&5]);
        assert_eq!(5, i.program.loop_map[&0]);
        assert_eq!(6, i.program.loop_map[&9]);
        assert_eq!(9, i.program.loop_map[&6]);
        assert_eq!(7, i.program.loop_map[&8]);
        assert_eq!(8, i.program.loop_map[&7]);
        assert_eq!(10, i.program.loop_map[&11]);
        assert_eq!(11, i.program.loop_map[&10]);
    }

    #[test]
    fn test_hello_world() {
        let src = include_str!("..\\samples\\helloworld.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let mut i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("Hello World!\n", io_clone.output());
    }

    #[test]
    fn test_666() {
        let src = include_str!("..\\samples\\666.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let mut i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("666\n", io_clone.output());
    }

    #[test]
    fn test_pi4() {
        let src = include_str!("..\\samples\\pi4.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let mut i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("3.141\n", io_clone.output());
    }

    #[test]
    fn test_sierpinski() {
        let src = include_str!("..\\samples\\sierpinski.bf");
        let out = include_str!("..\\samples\\out\\sierpinski.txt");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let mut i = VM::new_with_io(p, io);

        i.run();
        assert_eq!(out, io_clone.output());
    }

    #[test]
    fn test_fib11() {
        let src = include_str!("..\\samples\\fib11.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let mut i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89", io_clone.output());
    }
}
