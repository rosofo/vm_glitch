#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Opcode {
    /// Allow for extra space in the bytecode
    Noop,
    /// Copy from `i` to `j` in both the bytecode and the audio buffer
    Copy,
    /// Flip the data at `i`. For the bytecode this is a binary NOT, for samples this is `1 - sample`
    Flip,
    /// Jump from `i` to `j`. For the bytecode this changes the PC, for the buffer it causes an audible skip
    Jump,
    /// Copy sample `i` from the audio buffer into the bytecode. If there are multiple channels this will be the product.
    Sample,
    /// Swap chunk `i` and `j` in the audio buffer and byte `i` for `j` in the bytecode.
    Swap,
}

#[derive(Debug)]
pub enum Op {
    Copy(usize, usize),
    Flip(usize),
    Jump(usize),
    Sample(usize),
    Swap(usize, usize),
}

impl Opcode {
    // TODO: Pretty sure theres a crate that
    //       generates this for us
    pub fn parse(num: u8) -> Result<Opcode, ()> {
        if num == Opcode::Noop as u8 {
            Ok(Opcode::Noop)
        } else if num == Opcode::Copy as u8 {
            Ok(Opcode::Copy)
        } else if num == Opcode::Flip as u8 {
            Ok(Opcode::Flip)
        } else if num == Opcode::Jump as u8 {
            Ok(Opcode::Jump)
        } else if num == Opcode::Sample as u8 {
            Ok(Opcode::Sample)
        } else if num == Opcode::Swap as u8 {
            Ok(Opcode::Swap)
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub enum Op2 {
    Unknown,
    Noop,
    Copy(u8, u8),
    Flip(u8),
    Jump(u8),
    Sample(u8),
    Swap(u8, u8),
}
