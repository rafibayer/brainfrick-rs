/// Brainfuck VM Instructions
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Instruction {
    // Standard Brainfuck instructions
    /// Commands: `>` | '<'
    Shift(isize),
    /// Commands: `+` | '-'
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
    /// Compiler-internal placeholder
    NoOp,
    /// Clear the current cell
    Clear,
    /// Alter the cell specified by its offset relative to the current cell
    /// by the current cells value times `mul`
    CopyClear { mul: u8, offset: isize },
}

impl TryFrom<char> for Instruction {
    type Error = ();

    /// Convert value into an `Instruction`.
    /// Returns `Err(())` if value is not a valid `Instruction`.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Instruction::*;

        // Convert each source token into it's corresponding
        // Instruction, or Err(()) if it is not a valid brainfuck instruction
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
