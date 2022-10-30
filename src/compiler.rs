use std::{collections::HashMap, fmt::Display};

use crate::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub loop_map: HashMap<usize, usize>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut indent = 0;
        let mut buf = String::new();
        for ins in &self.instructions {
            if *ins == Instruction::End {
                indent -= 1;
            }

            buf.push_str(&format!("{}{:?},\n", "\t".repeat(indent), *ins));

            if *ins == Instruction::Loop {
                indent += 1;
            }
        }

        write!(f, "{}", buf)
    }
}

pub fn compile(src: &str) -> Program {
    // clean and parse input
    let mut instructions = src
        .chars()
        .map(Instruction::try_from)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for optimizer in get_optimizers() {
        instructions = optimizer.optimize(instructions);
    }

    // Loop instructions, must happen last because
    // optimizers can change position of loop instructions
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

    assert_eq!(0, stack.len());

    Program {
        instructions,
        loop_map,
    }
}

fn get_optimizers() -> Vec<OptimizerType> {
    use OptimizerType::*;
    vec![
        // contract repeated alts and shifts
        Contraction,
        // contract alt(0), NoOp, shift(0)
        NoOpReducer,
        // contract [-]
        ClearLoop,
        // contract single target copys
        CopyLoop,
    ]
}

pub trait Optimizer {
    fn optimize(&self, instructions: Vec<Instruction>) -> Vec<Instruction>;
}

enum OptimizerType {
    Contraction,
    ClearLoop,
    CopyLoop,
    NoOpReducer,
}

impl Optimizer for OptimizerType {
    fn optimize(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        match self {
            OptimizerType::Contraction => contraction_optimizer(instructions),
            OptimizerType::ClearLoop => clear_loop_optimizer(instructions),
            OptimizerType::CopyLoop => copy_loop_optimizer(instructions),
            OptimizerType::NoOpReducer => no_op_reducer(instructions),
        }
    }
}

fn contraction_optimizer(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut output = Vec::new();
    let mut input = instructions.drain(..).peekable();
    let mut next: Option<Instruction> = input.next();

    while let Some(cur) = next {
        match cur {
            Instruction::Shift(mut count) => {
                while let Some(Instruction::Shift(more)) = input.peek() {
                    count += *more;
                    input.next();
                }

                output.push(Instruction::Shift(count));
            }
            Instruction::Alt(mut count) => {
                while let Some(Instruction::Alt(more)) = input.peek() {
                    count += *more;
                    input.next();
                }

                output.push(Instruction::Alt(count));
            }
            other => output.push(other),
        }

        next = input.next();
    }

    output
}

fn clear_loop_optimizer(instructions: Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;
    let mut output: Vec<Instruction> = Vec::new();

    for instruction in instructions {
        output.push(instruction);

        if output.len() >= 3 {
            match output[output.len() - 3..] {
                [Loop, Alt(-1), End] => {
                    pop_n(&mut output, 3);
                    output.push(Clear);
                }
                _ => {}
            };
        }
    }

    output
}

fn copy_loop_optimizer(instructions: Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;
    let mut output = Vec::new();

    for instruction in instructions {
        output.push(instruction);

        if output.len() >= 6 {
            match output[output.len() - 6..] {
                [Loop, Alt(-1), Shift(off1), Alt(x), Shift(off2), End] if x > 0 && off1 == -off2 => {
                    pop_n(&mut output, 6);

                    output.push(Copy {
                        mul: x as u8,
                        offset: off1,
                    });
                    output.push(Clear);
                }
                [Loop, Shift(off1), Alt(x), Shift(off2), Alt(-1), End] if x > 0 && off1 == -off2 => {
                    pop_n(&mut output, 6);

                    output.push(Copy {
                        mul: x as u8,
                        offset: off1,
                    });
                    output.push(Clear);
                }
                _ => {}
            }
        }
    }

    output
}

fn no_op_reducer(instructions: Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;
    let mut output = vec![];

    for instruction in instructions {
        if !matches!(instruction, NoOp | Alt(0) | Shift(0)) {
            output.push(instruction);
        }
    }

    output
}

fn pop_n<T>(vec: &mut Vec<T>, n: usize) {
    let final_length = vec.len().saturating_sub(n);
    vec.truncate(final_length);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contraction_optimizer() {
        use Instruction::*;
        let input = vec![Shift(1), Shift(2), Shift(3), Shift(-2)];

        let output = contraction_optimizer(input);
        assert_eq!(vec![Shift(4)], output);
    }

    #[test]
    fn test_clear_loop_optimizer() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), End];

        let output = clear_loop_optimizer(input);
        assert_eq!(vec![Clear], output);
    }

    #[test]
    fn test_clear_loop_offset_optimizer() {
        use Instruction::*;
        let input = vec![Shift(1), Loop, Alt(-1), End, Alt(1)];

        let output = clear_loop_optimizer(input);
        assert_eq!(vec![Shift(1), Clear, Alt(1)], output);
    }

    #[test]
    fn test_copy_loop_optimizer_right() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), Shift(5), Alt(1), Shift(-5), End];

        let output = copy_loop_optimizer(input);
        assert_eq!(vec![Copy { mul: 1, offset: 5 }, Clear], output);
    }

    #[test]
    fn test_copy_loop_optimizer_left() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), Shift(-3), Alt(1), Shift(3), End];

        let output = copy_loop_optimizer(input);
        assert_eq!(vec![Copy { mul: 1, offset: -3 }, Clear], output);
    }

    #[test]
    fn test_copy_loop_mul() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), Shift(3), Alt(4), Shift(-3), End];

        let output = copy_loop_optimizer(input);
        assert_eq!(vec![Copy { mul: 4, offset: 3 }, Clear], output);
    }
}
