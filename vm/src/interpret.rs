use std::sync::{atomic::AtomicUsize, Arc};

use crate::{
    backend::{Backend, NoopBackend},
    op::{Op, Opcode},
    state::VmState,
};
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
    state: VmState,
    pub ui_counters: (Arc<AtomicUsize>, Arc<AtomicUsize>),
}

impl Vm {
    /// Run the Vm's current bytecode.
    ///
    /// Modifies the audio buffer and the bytecode simultaneously.
    pub fn run<B: Backend>(&mut self, bytecode: &mut [u8], backend: &mut B, self_modify: bool) {
        self.reset();
        while self.state.pc < bytecode.len() && self.state.total_for_run <= self.max_instructions {
            self.step(bytecode, backend, self_modify);
            self.notify();
        }
    }

    #[instrument(skip(self, bytecode, backend))]
    fn step<B: Backend>(&mut self, bytecode: &mut [u8], backend: &mut B, self_modify: bool) {
        #[cfg(feature = "tracing")]
        {
            tracy_client::plot!("PC", self.state.pc as f64);
            tracy_client::plot!("buf_idx", self.state.buf_index as f64);
            tracy_client::plot!("total_for_run", self.state.total_for_run as f64);
        }

        let op = self.parse_op(bytecode, REGISTER_COUNT);
        if let Some(op) = op {
            self.run_op(op, bytecode, backend, self_modify);
        }

        self.increment()
    }

    /// Parses the current [Op] and its args
    #[instrument(skip(self, bytecode))]
    fn parse_op(&mut self, bytecode: &mut [u8], registers: usize) -> Option<Op> {
        let byte = *bytecode.get(self.state.pc)?;
        if byte == Opcode::Copy as u8 || byte == Opcode::Swap as u8 {
            let i = *bytecode.get(self.state.pc + 1)? as usize;
            self.state.pc += 1;
            let j = *bytecode.get(self.state.pc + 1)? as usize;
            self.state.pc += 1;

            if byte == Opcode::Swap as u8 {
                return Some(Op::Swap(i % REGISTER_COUNT, j % REGISTER_COUNT));
            } else {
                return Some(Op::Copy(i % REGISTER_COUNT, j % REGISTER_COUNT));
            }
        } else {
            let i = *bytecode.get(self.state.pc + 1)? as usize;
            if byte == Opcode::Jump as u8 {
                self.state.pc += 1;
                return Some(Op::Jump(i % REGISTER_COUNT));
            } else if byte == Opcode::CopyFromSelf as u8 {
                let pc = self.state.pc;
                self.state.pc += 1;
                return Some(Op::Copy(pc % REGISTER_COUNT, i % REGISTER_COUNT));
            } else if byte == Opcode::Flip as u8 {
                self.state.pc += 1;
                return Some(Op::Flip(i % REGISTER_COUNT));
            } else if byte == Opcode::Sample as u8 {
                self.state.pc += 1;
                return Some(Op::Sample(i % REGISTER_COUNT));
            }
        }
        None
    }

    #[instrument(skip(self, bytecode, backend))]
    fn run_op<B: Backend>(
        &mut self,
        op: Op,
        bytecode: &mut [u8],
        backend: &mut B,
        self_modify: bool,
    ) {
        let chunk_size_bytecode = bytecode.len() / REGISTER_COUNT;
        match op {
            Op::Copy(from_idx, to_idx) => {
                #[cfg(feature = "tracing")]
                if self_modify {
                    tracy_client::plot!("bytecode Op::Copy", 1.0);
                    let chunk_start = from_idx * chunk_size_bytecode;
                    let chunk_end = chunk_start + chunk_size_bytecode;
                    bytecode.copy_within(chunk_start..chunk_end, to_idx * chunk_size_bytecode);
                }
                backend.run(bytecode, Op::Copy(from_idx, to_idx), &self.state);
            }
            Op::Jump(i) => {
                self.state.pc = i;
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Jump", 1.0);
                backend.run(bytecode, Op::Jump(i), &self.state);
            }
            Op::Sample(i) => {
                backend.run(bytecode, Op::Sample(i), &self.state);
            }
            Op::Swap(i, j) => {
                if self_modify {
                    for offset in 0..chunk_size_bytecode {
                        bytecode.swap(
                            (i * chunk_size_bytecode) + offset,
                            (j * chunk_size_bytecode) + offset,
                        );
                    }
                    #[cfg(feature = "tracing")]
                    tracy_client::plot!("bytecode Op::Swap", 1.0);
                }
                backend.run(bytecode, Op::Swap(i, j), &self.state);
            }
            _ => {}
        }
    }

    fn increment(&mut self) {
        self.state.pc += 1;
        self.state.total_for_run += 1;
        self.state.buf_index += 1;
    }

    fn notify(&self) {
        self.ui_counters
            .0
            .store(self.state.pc, std::sync::atomic::Ordering::Relaxed);
        self.ui_counters
            .1
            .store(self.state.buf_index, std::sync::atomic::Ordering::Relaxed);
    }

    /// Prepare for the next run
    fn reset(&mut self) {
        self.state.pc = 0;
        self.state.total_for_run = 0;
        self.state.buf_index = 0;
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            max_instructions: 512,
            state: VmState::default(),
            ui_counters: (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))),
        }
    }
}
