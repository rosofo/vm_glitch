use std::sync::{atomic::AtomicUsize, Arc};

use crate::op::{Op, Opcode};
use dasp::*;
use ring_buffer::Fixed;
use tracing::instrument;

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
    pub ui_counters: (Arc<AtomicUsize>, Arc<AtomicUsize>),
}

impl Vm {
    /// Run the Vm's current bytecode.
    ///
    /// Modifies the audio buffer and the bytecode simultaneously.
    pub fn run(&mut self, bytecode: &mut [u8], self_modify: bool) {
        self.reset();
        while self.pc < bytecode.len() && self.total_for_run <= self.max_instructions {
            self.step(bytecode, self_modify);
            self.notify();
        }
    }

    #[instrument(skip(self, bytecode))]
    fn step(&mut self, bytecode: &mut [u8], self_modify: bool) {
        #[cfg(feature = "tracing")]
        {
            tracy_client::plot!("PC", self.pc as f64);
            tracy_client::plot!("buf_idx", self.buf_index as f64);
            tracy_client::plot!("total_for_run", self.total_for_run as f64);
        }

        let op = self.parse_op(bytecode, REGISTER_COUNT);
        if let Some(op) = op {
            self.run_op(op, bytecode, self_modify);
        }

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

    #[instrument(skip(self, bytecode))]
    fn run_op(&mut self, op: Op, bytecode: &mut [u8], self_modify: bool) {
        let chunk_size_bytecode = bytecode.len() / REGISTER_COUNT;
        match op {
            Op::Copy(from_idx, to_idx) => {
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Copy", 1.0);

                if self_modify {
                    let chunk_start = from_idx * chunk_size_bytecode;
                    let chunk_end = chunk_start + chunk_size_bytecode;
                    bytecode.copy_within(chunk_start..chunk_end, to_idx * chunk_size_bytecode);
                }
            }
            Op::Jump(i) => {
                self.pc = i;
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Jump", 1.0);
            }
            Op::Sample(i) => {
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Sample", 1.0);
            }
            Op::Swap(i, j) => {
                if self_modify {
                    for offset in 0..chunk_size_bytecode {
                        bytecode.swap(
                            (i * chunk_size_bytecode) + offset,
                            (j * chunk_size_bytecode) + offset,
                        );
                    }
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

    fn notify(&self) {
        self.ui_counters
            .0
            .store(self.pc, std::sync::atomic::Ordering::Relaxed);
        self.ui_counters
            .1
            .store(self.buf_index, std::sync::atomic::Ordering::Relaxed);
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
            ui_counters: (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))),
        }
    }
}
