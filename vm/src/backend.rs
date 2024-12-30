use dasp::ring_buffer;
use numquant::linear;

use crate::{op::Op, state::VmState, REGISTER_COUNT};

pub trait Backend {
    fn run(&mut self, bytecode: &mut [u8], op: Op, vm_state: &VmState);
}

pub struct NoopBackend;

impl Backend for NoopBackend {
    fn run(&mut self, _bytecode: &mut [u8], _op: Op, _vm_state: &VmState) {}
}

impl Backend for ring_buffer::Fixed<Vec<[f32; 2]>> {
    fn run(&mut self, bytecode: &mut [u8], op: Op, vm_state: &VmState) {
        let chunk_size_audio = self.len() / REGISTER_COUNT;
        match op {
            Op::Copy(from_idx, to_idx) => {
                let chunk_start = from_idx * chunk_size_audio;
                let chunk_end = chunk_start + chunk_size_audio;
                for (i, frame) in (chunk_start..chunk_end).enumerate() {
                    let from_frame = *self.get(frame);
                    let to_frame = self.get_mut((to_idx * chunk_size_audio) + i);
                    to_frame[0] = from_frame[0];
                    to_frame[1] = from_frame[1];

                    #[cfg(feature = "tracing")]
                    tracy_client::plot!("Op::Copy", 1.0);
                }
            }
            Op::Sample(i) => {
                let frame = self.get(i);
                let mut sample = frame[0] + frame[1];
                sample /= self.len() as f32;
                bytecode[vm_state.pc] = linear::quantize(sample as f64, -1.0..1.0, 255);
                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Sample", 1.0);
            }
            Op::Swap(i, j) => {
                for offset in 0..chunk_size_audio {
                    let j_frame = *self.get((j * chunk_size_audio) + offset);
                    let i_frame = self.get_mut((i * chunk_size_audio) + offset);
                    let i_backup = *i_frame;
                    i_frame[0] = j_frame[0];
                    i_frame[1] = j_frame[1];
                    let j_frame = self.get_mut((j * chunk_size_audio) + offset);
                    j_frame[0] = i_backup[0];
                    j_frame[1] = i_backup[1];
                }

                #[cfg(feature = "tracing")]
                tracy_client::plot!("Op::Swap", 1.0);
            }
            _ => {}
        }

        let chans = self.get(vm_state.pc);
        let (left, right) = (chans[0], chans[1]);
        let chans = self.get_mut(vm_state.buf_index);
        chans[0] = left;
        chans[1] = right;
    }
}
