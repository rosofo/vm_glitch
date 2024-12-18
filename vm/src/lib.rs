pub mod op;
use op::{Op, Opcode};

pub type RawBuffer<'a, 'b> = &'a mut [&'b mut [f32]];

// TODO: Make this a fun resolution slider
const REGISTER_COUNT: usize = 16;

#[derive(Clone, Debug)]
pub struct Vm {
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
    pc: usize,
    /// The monotonically increasing index into the audio buffer as we run through instructions
    buf_index: usize,
    /// The total instructions/samples processed. Resets to 0 after each run.
    total_for_run: usize,
}

impl Vm {
    /// Run the Vm's current bytecode.
    ///
    /// Modifies the audio buffer and the bytecode simultaneously.
    pub fn run(&mut self, bytecode: &mut [u8], buf: RawBuffer, samples: usize) {
        self.reset();
        while self.pc < bytecode.len()
            && self.buf_index < samples
            && self.total_for_run <= self.max_instructions
        {
            self.step(bytecode, buf, samples);
        }
    }

    fn step(&mut self, bytecode: &mut [u8], buf: RawBuffer, samples: usize) {
        let op = self.parse_op(bytecode, REGISTER_COUNT);
        if let Some(op) = op {
            self.run_op(op, buf, samples);
        }

        self.increment()
    }

    /// Parses the current [Op] and its args
    fn parse_op(&mut self, bytecode: &mut [u8], registers: usize) -> Option<Op> {
        let byte = bytecode[self.pc];
        let bytecode_len = bytecode.len();
        if byte == Opcode::Copy as u8 && self.pc + 2 < bytecode_len {
            self.pc += 1;
            let i = bytecode[self.pc] as usize;
            self.pc += 1;
            let j = bytecode[self.pc] as usize;

            if i < registers && j < registers {
                return Some(Op::Copy(i, j));
            }
        } else if self.pc + 1 < bytecode_len {
            let i = bytecode[self.pc + 1] as usize;

            if i < bytecode_len {
                if byte == Opcode::Jump as u8 {
                    self.pc += 1;
                    return Some(Op::Jump(i));
                } else if byte == Opcode::Flip as u8 {
                    self.pc += 1;
                    return Some(Op::Flip(i));
                } else if byte == Opcode::Sample as u8 {
                    self.pc += 1;
                    return Some(Op::Sample(i));
                }
            }
        }
        None
    }

    fn run_op(&mut self, op: Op, buf: RawBuffer, samples: usize) {
        let chunk_size = samples/REGISTER_COUNT;
        match op {
            Op::Copy(i, j) => {
                for chan in buf.iter_mut() {
                    let chunk_start = i * chunk_size;
                    let chunk_end = chunk_start + chunk_size;
                    chan.copy_within(chunk_start..chunk_end, j*chunk_size);
                }
            }
            Op::Jump(i) => {
                self.pc = i;
            }
            _ => {}
        }
    }

    fn increment(&mut self) {
        self.pc += 1;
        self.total_for_run += 1;
        self.buf_index += 1;
    }

    /// Prepare for the next run
    fn reset(&mut self) {
        self.pc = 0;
        self.total_for_run = 0;
        self.buf_index = 0;
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            max_instructions: 512,
            buf_index: 0,
            pc: 0,
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
            mut bytecode in prop::collection::vec(prop::bits::u8::ANY, 512),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
        ) {
            let mut buf_ = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
            let mut vm = Vm::default();
            for _ in 0..513 {
                vm.run(&mut bytecode, buf_.as_mut_slice(), 512);
            }
        }
    }

    #[test]
    fn test_jumping() {
        let mut buf = [-1.0f32, 0.2, 0.4, -0.3];
        let mut vm = Vm::default();
        let mut bytecode = vec![Opcode::Jump as u8, 2, 0, 0];
        vm.step(&mut bytecode, &mut [&mut buf], 4);

        assert_eq!(vm.pc, 3);
    }

    proptest! {
    #[test]
    fn test_copying_alters_one_byte_max(
            mut bytecode in prop::collection::vec(0..128u8, 511),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
    ) {
        bytecode.insert(0,Opcode::Copy as u8);
        let orig = bytecode.clone();
        let mut buf_ = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
        let mut vm = Vm::default();
        vm.step(&mut bytecode, &mut buf_[..], 512);

        let diff_bytes = bytecode.iter().zip(orig.iter()).filter(|(a, b)| a != b).count();
        assert!(diff_bytes == 0 || diff_bytes == 1);
    }

    #[test]
    fn test_noops_dont_change_bytecode_or_buffer(
            mut bytecode in prop::collection::vec(0u8..1, 511),
            mut buf in prop::collection::vec(prop::collection::vec(-1.0..1.0f32, 512), 2)
    ) {
        let orig = bytecode.clone();
        let buf_ = buf.clone();
        let mut buf_slice = buf.iter_mut().map(|s| &mut s[..]).collect::<Vec<_>>();
        let mut vm = Vm::default();
        vm.step(&mut bytecode, &mut buf_slice[..], 512);

        assert_eq!(bytecode, orig);
        assert_eq!(buf, buf_);
    }

    }
}
