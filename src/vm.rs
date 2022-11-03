use std::fmt::Display;

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
    /// Compiled Brainfuck program
    program: Program,

    /// Program Memory
    data: Box<[u8; MEM]>,

    /// Memory pointer
    ptr: usize,

    /// InputOutput implementation
    io: IO,
}

/// Pretty view of brainfuck VM state
impl<IO: InputOutput> Display for VM<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("{\n");
        buf.push_str(&format!("\tptr: {}\n", self.ptr));
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
    /// Create a new Brainfuck VM to execute the given Program.
    /// Configured to use Stdin and Stdout.
    pub fn new(program: Program) -> Self {
        VM::new_with_io(program, StdIO {})
    }
}

impl<IO: InputOutput> VM<IO> {
    /// Create a new Brainfuck VM to execute the given Program.
    /// Configured to use the given `IO` for input-output.
    pub fn new_with_io(program: Program, io: IO) -> Self {
        VM {
            program,
            data: Box::new([0; MEM]),
            ptr: 0,
            io,
        }
    }

    /// Runs the VM
    pub fn run(mut self) {
        let mut instruction_ptr = 0;

        while instruction_ptr < self.program.instructions.len() {
            // current instruction to execute
            let instruction = &self.program.instructions[instruction_ptr];

            // instruction implementations
            match instruction {
                Instruction::Shift(count) => self.ptr = (self.ptr as isize + count) as usize,
                Instruction::Alt(amount) => {
                    self.data[self.ptr] = match *amount >= 0 {
                        true => self.data[self.ptr].wrapping_add(*amount as u8),
                        false => self.data[self.ptr].wrapping_sub(-amount as u8),
                    };
                }
                Instruction::Out => self.io.print(self.data[self.ptr]),
                Instruction::In => self.data[self.ptr] = self.io.getch(),
                Instruction::Loop => {
                    if self.data[self.ptr] == 0u8 {
                        instruction_ptr = self.program.loop_map[instruction_ptr];
                    }
                }
                Instruction::End => {
                    if self.data[self.ptr] != 0u8 {
                        instruction_ptr = self.program.loop_map[instruction_ptr];
                    }
                }
                Instruction::Clear => {
                    // optimized version of [-]
                    self.data[self.ptr] = 0u8;
                }
                Instruction::CopyClear { mul, offset } => {
                    let target_d_ptr = ((self.ptr as isize + offset) as usize) % MEM;
                    let new_value = self.data[target_d_ptr].wrapping_add(self.data[self.ptr] * mul);
                    self.data[self.ptr] = 0u8;
                    self.data[target_d_ptr] = new_value;
                }
                Instruction::NoOp => {}
            };

            instruction_ptr += 1;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;

    use crate::{compiler::compile, io::TestIO};

    use super::*;

    // todo: this test will break if compiler
    // change the braces in the output program
    #[test]
    fn test_bracket_matching() {
        let p = compile("[->.<][[]][]");
        let i = VM::new(p);

        assert_eq!(12, i.program.loop_map.len());
        assert_eq!(0, i.program.loop_map[5]);
        assert_eq!(5, i.program.loop_map[0]);
        assert_eq!(6, i.program.loop_map[9]);
        assert_eq!(9, i.program.loop_map[6]);
        assert_eq!(7, i.program.loop_map[8]);
        assert_eq!(8, i.program.loop_map[7]);
        assert_eq!(10, i.program.loop_map[11]);
        assert_eq!(11, i.program.loop_map[10]);
    }

    #[test]
    fn test_hello_world() {
        let src = include_str!("..\\samples\\helloworld.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("Hello World!\n", io_clone.output());
    }

    #[test]
    fn test_666() {
        let src = include_str!("..\\samples\\666.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("666\n", io_clone.output());
    }

    #[test]
    fn test_pi4() {
        let src = include_str!("..\\samples\\pi4.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let i = VM::new_with_io(p, io);

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
        let i = VM::new_with_io(p, io);

        i.run();
        assert_eq!(out, io_clone.output());
    }

    #[test]
    fn test_fib11() {
        let src = include_str!("..\\samples\\fib11.bf");
        let p = compile(src);
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let i = VM::new_with_io(p, io);

        i.run();
        assert_eq!("1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89", io_clone.output());
    }
}
