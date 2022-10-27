use crate::io::{InputOutput, StdIO};
use std::collections::HashMap;

/// Standard Brainfuck instructions
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Instruction {
    /// Command: `>`
    Right,
    /// Command: `<`
    Left,
    /// Command: `+`
    Inc,
    /// Command: `-`
    Dec,
    /// Command: `.`
    Out,
    /// Command: `,`
    In,
    /// Command: `[`
    Loop,
    /// Command: `]`
    End,
}

impl TryFrom<char> for Instruction {
    type Error = ();

    /// Convert value into an `Instruction`.
    /// Returns `Err(())` if value is not a valid `Instruction`.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Instruction::*;

        Ok(match value {
            '>' => Right,
            '<' => Left,
            '+' => Inc,
            '-' => Dec,
            '.' => Out,
            ',' => In,
            '[' => Loop,
            ']' => End,
            _ => return Err(()),
        })
    }
}

/// Default memory size for interpreter.
const MEM: usize = 30_000;

/// Brainfuck Interpreter.
#[derive(Debug)]
pub struct Interpreter<IO: InputOutput> {
    /// Program Instructions
    instructions: Vec<Instruction>,
    /// Program Memory
    data: [u8; MEM],

    /// Instruction Pointer
    in_ptr: usize,

    /// Memory pointer
    d_ptr: usize,

    /// Pre-computed instruction pointers
    /// to match loop instructions.
    loop_map: HashMap<usize, usize>,

    /// InputOutput implementation
    io: IO,
}

impl Interpreter<StdIO> {
    pub fn new(src: &str) -> Self {
        Interpreter::new_with_io(src, StdIO {})
    }
}

impl<IO: InputOutput> Interpreter<IO> {
    pub fn new_with_io(src: &str, io: IO) -> Self {
        // clean and parse input
        let instructions = src
            .chars()
            .map(Instruction::try_from)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        // Loop instructions
        let mut loop_map = HashMap::new();
        let mut stack = Vec::new();
        for (ptr, ins) in instructions.iter().enumerate() {
            match *ins {
                Instruction::Loop => stack.push(ptr),
                Instruction::End => {
                    let open = stack.pop().unwrap();
                    loop_map.insert(open, ptr);
                    loop_map.insert(ptr, open);
                }
                _ => {}
            }
        }

        Interpreter {
            instructions,
            data: [0; MEM],
            in_ptr: 0,
            d_ptr: 0,
            loop_map,
            io,
        }
    }

    pub fn run(mut self) {
        loop {
            // program ends when we reach the end of
            // the instructions
            if self.in_ptr >= self.instructions.len() {
                return;
            }

            // unless we encounter a bracket, we will go to the next
            // instruciton after this one
            let mut next_ins_ptr = self.in_ptr + 1;

            // instruction implementations
            let ins = self.instructions[self.in_ptr];
            match ins {
                Instruction::Right => self.d_ptr += 1,
                Instruction::Left => self.d_ptr -= 1,
                Instruction::Inc => self.data[self.d_ptr] += 1,
                Instruction::Dec => self.data[self.d_ptr] -= 1,
                Instruction::Out => self.io.print(self.data[self.d_ptr]),
                Instruction::In => self.data[self.d_ptr] = self.io.getch(),
                Instruction::Loop => {
                    if self.data[self.d_ptr] == 0u8 {
                        next_ins_ptr = self.loop_map[&self.in_ptr] + 1;
                    }
                }
                Instruction::End => {
                    if self.data[self.d_ptr] != 0u8 {
                        next_ins_ptr = self.loop_map[&self.in_ptr] + 1;
                    }
                }
            };

            self.in_ptr = next_ins_ptr;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;

    use crate::io::TestIO;

    use super::*;

    #[test]
    fn test_parse() {
        use Instruction::*;

        let src = "c|om&ment   a [->+<] comment";
        let i = Interpreter::new(src);
        assert_eq!(vec![Loop, Dec, Right, Inc, Left, End], i.instructions);
    }

    #[test]
    fn test_bracket_matching() {
        let i = Interpreter::new("[->+<][[]][]");

        assert_eq!(8, i.loop_map.len());
        assert_eq!(0, i.loop_map[&5]);
        assert_eq!(5, i.loop_map[&0]);
        assert_eq!(6, i.loop_map[&9]);
        assert_eq!(9, i.loop_map[&6]);
        assert_eq!(7, i.loop_map[&8]);
        assert_eq!(8, i.loop_map[&7]);
        assert_eq!(10, i.loop_map[&11]);
        assert_eq!(11, i.loop_map[&10]);
    }

    #[test]
    fn test_hello_world() {
        let src = include_str!("..\\samples\\helloworld.bf");
        let io = Rc::new(TestIO::new(""));
        let io_clone = io.clone();
        let i = Interpreter::new_with_io(src, io);

        i.run();
        assert_eq!("Hello World!\n", io_clone.output());
    }
}
