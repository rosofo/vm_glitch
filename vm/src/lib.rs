pub mod op;
use op::{Op, Opcode};

pub type RawBuffer<'a, 'b> = &'a mut [&'b mut [f32]];

#[derive(Clone, Debug)]
pub struct Vm {
    pub bytecode: Vec<u8>,
    /// The maximum number of instructions to run.
    ///
    /// When this is reached the VM will halt early to avoid ever blocking/hanging the audio thread.
    max_instructions: usize,
    /// The current index in both the bytecode and the audio buffer
    ///
    /// Functions as the Program Counter for the bytecode, and controls sample output ordering.
    /// This is incremented after each instruction and resets to 0 after a run.
    ///
    /// Can be modified by an [Op::Jump]
    index: usize,
    /// The monotonically increasing index into the audio buffer as we run through instructions
    ///
    /// TODO this is hella confusing, clarify semantics
    buf_index: usize,
    /// The total instructions/samples processed. Resets to 0 after each run.
    total_for_run: usize,
}

impl Vm {
    /// Run the Vm's current bytecode.
    ///
    /// Modifies the audio buffer and the bytecode simultaneously.
    pub fn run(&mut self, buf: RawBuffer, samples: usize) {
        self.reset();
        while self.index < self.bytecode.len()
            && self.buf_index < samples
            && self.total_for_run <= self.max_instructions
        {
            self.step(buf, samples);
        }
    }

    fn step(&mut self, buf: RawBuffer, samples: usize) {
        let byte = self.bytecode[self.index];
        let op = self.parse_op(byte, samples);
        if let Some(op) = op {
            self.run_op(op, buf);
        }

        for chan in buf.iter_mut() {
            chan[self.buf_index] = chan[self.index];
        }
        self.increment()
    }

    /// Parses the current [Op] and its args and verifies that the operation is possible
    fn parse_op(&mut self, byte: u8, sample_len: usize) -> Option<Op> {
        let bytecode_len = self.bytecode.len();
        if byte == Opcode::Copy as u8 && self.index + 2 < bytecode_len {
            self.index += 1;
            let i = self.bytecode[self.index] as usize;
            self.index += 1;
            let j = self.bytecode[self.index] as usize;

            if i < bytecode_len && i < sample_len && j < bytecode_len && j < sample_len {
                return Some(Op::Copy(i, j));
            }
        } else if self.index + 1 < bytecode_len {
            let i = self.bytecode[self.index + 1] as usize;

            if i < bytecode_len && i < sample_len {
                if byte == Opcode::Jump as u8 {
                    self.index += 1;
                    return Some(Op::Jump(i));
                } else if byte == Opcode::Flip as u8 {
                    self.index += 1;
                    return Some(Op::Flip(i));
                } else if byte == Opcode::Sample as u8 {
                    self.index += 1;
                    return Some(Op::Sample(i));
                }
            }
        }
        None
    }

    fn run_op(&mut self, op: Op, buf: RawBuffer) {
        match op {
            Op::Copy(i, j) => {
                self.bytecode[j] = self.bytecode[i];
                for chan in buf.iter_mut() {
                    chan[j] = chan[i];
                }
            }
            Op::Jump(i) => {
                self.index = i;
            }
            _ => {}
        }
    }

    fn increment(&mut self) {
        self.index += 1;
        self.total_for_run += 1;
        self.buf_index += 1;
    }

    /// Prepare for the next run
    fn reset(&mut self) {
        self.index = 0;
        self.total_for_run = 0;
        self.buf_index = 0;
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            bytecode: vec![0u8; 512],
            max_instructions: 512,
            buf_index: 0,
            index: 0,
            total_for_run: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::{op::Opcode, Vm};

    proptest! {
        #[test]
        fn test_never_panics(
            bytecode in prop::collection::vec(prop::bits::u8::ANY, 512),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
        ) {
            let mut buf_ = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
            let mut vm = Vm::default();
            vm.bytecode = bytecode;
            for _ in 0..513 {
                vm.run(buf_.as_mut_slice(), 512);
            }
        }
    }

    #[test]
    fn test_jumping() {
        let mut buf = [-1.0f32, 0.2, 0.4, -0.3];
        let mut vm = Vm::default();
        let bytecode = vec![Opcode::Jump as u8, 2, 0, 0];
        vm.bytecode = bytecode.clone();
        vm.run(&mut [&mut buf], 4);

        assert_eq!(bytecode, vm.bytecode);
        assert_eq!(vm.index, 4); // bounds check fails at 4 and it returns
    }

    proptest! {
    #[test]
    fn test_copying_alters_one_byte_max(
            mut bytecode in prop::collection::vec(0..128u8, 511),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
    ) {
        bytecode.insert(0,Opcode::Copy as u8);
        let mut buf_ = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
        let mut vm = Vm::default();
        vm.bytecode = bytecode.clone();
        vm.step(&mut buf_[..], 512);

        let diff_bytes = bytecode.iter().zip(vm.bytecode.iter()).filter(|(a, b)| a != b).count();
        assert!(diff_bytes == 0 || diff_bytes == 1);
    }

    #[test]
    fn test_noops_dont_change_bytecode_or_buffer(
            mut bytecode in prop::collection::vec(0u8..1, 511),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
    ) {
        let mut buf_ = buf.clone();
        let mut buf_slice = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
        let mut vm = Vm::default();
        vm.bytecode = bytecode.clone();
        vm.step(&mut buf_slice[..], 512);

        assert_eq!(bytecode, vm.bytecode);
        assert_eq!(buf, buf_);
    }

    }
}