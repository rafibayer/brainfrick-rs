/// Brainfuck VM Instructions
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    /// Command: `>` | '<'
    Shift(isize),
    /// Command: `+` | '-'
    Alt(i16),
    /// Command: `.`
    Out,
    /// Command: `,`
    In,
    /// Command: `[`
    Loop,
    /// Command: `]`
    End,

    // Optimized instructions used by the compiler
    /// NoOp: compiler placeholder
    NoOp,
    Clear,
    Copy {
        mul: u8,
        offset: isize,
    },
}

impl TryFrom<char> for Instruction {
    type Error = ();

    /// Convert value into an `Instruction`.
    /// Returns `Err(())` if value is not a valid `Instruction`.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Instruction::*;

        Ok(match value {
            '>' => Shift(1),
            '<' => Shift(-1),
            '+' => Alt(1),
            '-' => Alt(-1),
            '.' => Out,
            ',' => In,
            '[' => Loop,
            ']' => End,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_parse() {
    //     use Instruction::*;

    //     let src = "c|om&ment   a [->+<] comment";
    //     let i = VM::new(src);
    //     assert_eq!(vec![Loop, Dec, Right, Inc, Left, End], i.instructions);
    // }
}
