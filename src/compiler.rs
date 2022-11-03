use std::fmt::Display;

use crate::instruction::Instruction;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CompilerStats {
    input_instructions: usize,
    output_instructions: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub loop_map: Vec<usize>,
    stats: CompilerStats,
}

/// Pretty display for Program
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

        write!(f, "{}{:?}", buf, self.stats)
    }
}

pub fn compile(src: &str) -> Program {
    // clean and parse input
    let mut instructions = src
        .chars()
        .map(Instruction::try_from)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let input_instructions = instructions.len();

    // apply each optmiziation to the instructions
    for optimizer in get_optimizers() {
        instructions = optimizer.optimize(instructions);
    }

    let output_instructions = instructions.len();

    // match loop instructions, must happen last because
    // optimizers can change position of loop instructions.
    // we could obviously use a map (even with identity hash) here to
    // save memory, but the perf gain of using a vec is massive (-35% time).
    let mut loop_map = vec![0; instructions.len()];
    let mut stack = Vec::new();
    for (ptr, ins) in instructions.iter().enumerate() {
        match *ins {
            Instruction::Loop => stack.push(ptr),
            Instruction::End => {
                let open = stack.pop().unwrap();
                loop_map[open] = ptr;
                loop_map[ptr] = open;
            }
            _ => {}
        }
    }

    // ensure our program always has balanced loops
    assert_eq!(0, stack.len());

    // Return the final compiled Program
    Program {
        instructions,
        loop_map,
        stats: CompilerStats {
            input_instructions,
            output_instructions,
        },
    }
}

/// Returns optimizers to apply to program and their ordering
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
    // enum dispatch to apply optimization function on source instructions
    fn optimize(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        match self {
            OptimizerType::Contraction => contraction_optimizer(instructions),
            OptimizerType::ClearLoop => clear_loop_optimizer(instructions),
            OptimizerType::CopyLoop => copy_loop_optimizer(instructions),
            OptimizerType::NoOpReducer => no_op_reducer(instructions),
        }
    }
}

/// Replace consecutive Shift and Alt instructions with single instructions.
fn contraction_optimizer(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut output = Vec::new();
    let mut input = instructions.drain(..).peekable();
    let mut next: Option<Instruction> = input.next();

    while let Some(cur) = next {
        match cur {
            // ex: ">><>>" -> Shift(3)
            Instruction::Shift(mut count) => {
                while let Some(Instruction::Shift(more)) = input.peek() {
                    count += *more;
                    input.next();
                }

                output.push(Instruction::Shift(count));
            }
            // ex: "+--+-" -> Alt(-1)
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

/// Replace loops to clear the current cell with Clear instructions
fn clear_loop_optimizer(instructions: Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;
    let mut output: Vec<Instruction> = Vec::new();

    for instruction in instructions {
        output.push(instruction);

        if output.len() >= 3 {
            // ex: "[-]" -> Clear
            if let [Loop, Alt(-1), End] = output[output.len() - 3..] {
                remove_n(&mut output, 3);
                output.push(Clear);
            };
        }
    }

    output
}

/// Replace copy-to/multiply loops with CopyClear instructions
fn copy_loop_optimizer(instructions: Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;
    let mut output = Vec::new();

    for instruction in instructions {
        output.push(instruction);

        if output.len() >= 6 {
            match output[output.len() - 6..] {
                // ex: "[->>>++<<<]" -> CopyClear { mul: 2, offset: 3 }
                [Loop, Alt(-1), Shift(off1), Alt(x), Shift(off2), End] if x > 0 && off1 == -off2 => {
                    remove_n(&mut output, 6);

                    output.push(CopyClear {
                        mul: x as u8,
                        offset: off1,
                    });
                }
                // ex: "[>>---<<+-]" -> CopyClear { mul: -3, offset: 2 }
                [Loop, Shift(off1), Alt(x), Shift(off2), Alt(-1), End] if x > 0 && off1 == -off2 => {
                    remove_n(&mut output, 6);

                    output.push(CopyClear {
                        mul: x as u8,
                        offset: off1,
                    });
                }
                _ => {}
            }
        }
    }

    output
}

/// Remove NoOp, Alt(0), Shift(0) instructions
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

/// Helper to remove up to the last n instructions from a `Vec<T>`
fn remove_n<T>(vec: &mut Vec<T>, n: usize) {
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
        assert_eq!(vec![CopyClear { mul: 1, offset: 5 }], output);
    }

    #[test]
    fn test_copy_loop_optimizer_left() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), Shift(-3), Alt(1), Shift(3), End];

        let output = copy_loop_optimizer(input);
        assert_eq!(vec![CopyClear { mul: 1, offset: -3 }], output);
    }

    #[test]
    fn test_copy_loop_mul() {
        use Instruction::*;
        let input = vec![Loop, Alt(-1), Shift(3), Alt(4), Shift(-3), End];

        let output = copy_loop_optimizer(input);
        assert_eq!(vec![CopyClear { mul: 4, offset: 3 }], output);
    }
}
