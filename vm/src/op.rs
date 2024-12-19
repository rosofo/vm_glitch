
#[derive(PartialEq, Eq, PartialOrd, Ord)]
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
}

pub enum Op {
    Copy(usize, usize),
    Flip(usize),
    Jump(usize),
    Sample(usize),
}
