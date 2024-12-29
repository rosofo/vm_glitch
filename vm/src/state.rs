#[derive(Clone, Debug, Default)]
pub struct VmState {
    /// The current index in both the bytecode and the audio buffer
    ///
    /// Functions as the Program Counter for the bytecode, and controls sample output ordering.
    /// This is incremented after each instruction and resets to 0 after a run.
    ///
    /// Can be modified by an [Op::Jump]
    pub pc: usize,
    /// The monotonically increasing index into the audio buffer as we run through instructions
    pub buf_index: usize,
    /// The total instructions/samples processed. Resets to 0 after each run.
    pub total_for_run: usize,
}
