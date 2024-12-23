pub mod op;
use dasp::*;
use numquant::linear;
use op::{Op, Opcode};
use ring_buffer::Fixed;
use tracing::{debug, instrument, trace};

pub type RawBuffer<'a> = &'a mut Fixed<Vec<[f32; 2]>>;

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
    pub fn run(&mut self, bytecode: &mut [u8], buf: RawBuffer) {
        self.reset();
        while self.pc < bytecode.len()
            && self.buf_index < buf.len()
            && self.total_for_run <= self.max_instructions
        {
            self.step(bytecode, buf);
        }
    }

    #[instrument(skip(self, bytecode, buf))]
    fn step(&mut self, bytecode: &mut [u8], buf: RawBuffer) {
        #[cfg(feature = "tracing")]
        {
            tracy_client::plot!("PC", self.pc as f64);
            tracy_client::plot!("buf_idx", self.buf_index as f64);
            tracy_client::plot!("total_for_run", self.total_for_run as f64);
        }

        let op = self.parse_op(bytecode, REGISTER_COUNT);
        if let Some(op) = op {
            self.run_op(op, bytecode, buf);
        }

        let chans = buf.get(self.pc);
        let (left, right) = (chans[0], chans[1]);
        let chans = buf.get_mut(self.buf_index);
        chans[0] = left;
        chans[1] = right;

        self.increment()
    }

    /// Parses the current [Op] and its args
    #[instrument(skip(self, bytecode))]
    fn parse_op(&mut self, bytecode: &mut [u8], registers: usize) -> Option<Op> {
        let byte = *bytecode.get(self.pc)?;
        if byte == Opcode::Copy as u8 || byte == Opcode::Swap as u8 {
            let i = *bytecode.get(self.pc + 1)? as usize;
            self.pc += 1;
            let j = *bytecode.get(self.pc + 1)? as usize;
            self.pc += 1;

            if i >= registers || j >= registers {
                return None;
            }

            if byte == Opcode::Swap as u8 {
                return Some(Op::Swap(i, j));
            } else {
                return Some(Op::Copy(i, j));
            }
        } else {
            let i = *bytecode.get(self.pc + 1)? as usize;
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
        None
    }

    #[instrument(skip(self, bytecode, buf))]
    fn run_op(&mut self, op: Op, bytecode: &mut [u8], buf: RawBuffer) {
        let chunk_size_audio = buf.len() / REGISTER_COUNT;
        let chunk_size_bytecode = bytecode.len() / REGISTER_COUNT;
        match op {
            Op::Copy(from_idx, to_idx) => {
                let chunk_start = from_idx * chunk_size_audio;
                let chunk_end = chunk_start + chunk_size_audio;
                for (i, frame) in (chunk_start..chunk_end).enumerate() {
                    let from_frame = *buf.get(frame);
                    let to_frame = buf.get_mut((to_idx * chunk_size_audio) + i);
                    to_frame[0] = from_frame[0];
                    to_frame[1] = from_frame[1];

                    #[cfg(feature = "tracing")]
                    tracy_client::plot!("Op::Copy", 1.0);
                }

                let chunk_start = from_idx * chunk_size_bytecode;
                let chunk_end = chunk_start + chunk_size_bytecode;
                bytecode.copy_within(chunk_start..chunk_end, to_idx * chunk_size_bytecode);
            }
            Op::Jump(i) => {
                self.pc = i;
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Jump", 1.0);
            }
            Op::Sample(i) => {
                let frame = buf.get(i);
                let mut sample = frame[0] + frame[1];
                sample /= buf.len() as f32;
                bytecode[self.pc] = linear::quantize(sample as f64, -1.0..1.0, 255);
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Sample", 1.0);
            }
            Op::Swap(i, j) => {
                for offset in 0..chunk_size_audio {
                    let j_frame = *buf.get((j * chunk_size_audio) + offset);
                    let i_frame = buf.get_mut((i * chunk_size_audio) + offset);
                    let i_backup = *i_frame;
                    i_frame[0] = j_frame[0];
                    i_frame[1] = j_frame[1];
                    let j_frame = buf.get_mut((j * chunk_size_audio) + offset);
                    j_frame[0] = i_backup[0];
                    j_frame[1] = i_backup[1];
                }

                for offset in 0..chunk_size_bytecode {
                    bytecode.swap(
                        (i * chunk_size_bytecode) + offset,
                        (j * chunk_size_bytecode) + offset,
                    );
                }
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Swap", 1.0);
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
    use dasp::ring_buffer::Fixed;
    use proptest::prelude::*;

    use crate::{op::Opcode, Vm};

    proptest! {
        #[test]
        fn test_never_panics(
            mut bytecode in prop::collection::vec(prop::bits::u8::ANY, 512),
            buf in prop::collection::vec(prop::array::uniform2(-1.0..1.0f32), 512)
        ) {
            let mut buf = Fixed::from(buf);
            let mut vm = Vm::default();
            for _ in 0..513 {
                vm.run(&mut bytecode, &mut buf);
            }
        }
    }

    #[test]
    fn test_jumping() {
        let mut buf = Fixed::from(vec![[-1.0f32, 0.2], [0.4, -0.3]]);
        let mut vm = Vm::default();
        let mut bytecode = vec![Opcode::Jump as u8, 2, 0, 0];
        vm.step(&mut bytecode, &mut buf);

        assert_eq!(vm.pc, 3);
    }

    proptest! {

    #[test]
    fn test_noops_dont_change_bytecode_or_buffer(
            mut bytecode in prop::collection::vec(0u8..1, 511),
            buf in prop::collection::vec(prop::array::uniform2(-1.0..1.0f32), 512)
    ) {
        let mut buf = Fixed::from(buf);
        let orig = bytecode.clone();
        let buf_ = buf.clone();
        let mut vm = Vm::default();
        vm.step(&mut bytecode, &mut buf);

        assert_eq!(bytecode, orig);
        assert_eq!(buf, buf_);
    }

    }
}
