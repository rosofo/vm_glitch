use std::thread::spawn;

use tracing::trace;
use triple_buffer::{triple_buffer, Input, Output};
use vm::{backend::NoopBackend, interpret::Vm};

pub struct BytecodeThread {
    bytecode: Vec<u8>,
    vm: Vm,
    size: usize,
}
pub struct BytecodeComms {
    pub bc_in: Input<Vec<u8>>,
    pub ui_out: Output<Vec<u8>>,
    pub audio_out: Output<Vec<u8>>,
    pub video_out: Output<Vec<u8>>,
}

impl BytecodeThread {
    pub fn new(size: usize) -> Self {
        Self {
            bytecode: vec![0u8; size],
            vm: Vm::default(),
            size,
        }
    }
    pub fn spawn(mut self) -> BytecodeComms {
        let (from_ui_in, mut from_ui_out) = triple_buffer(&vec![0u8; self.size]);
        let (mut to_ui_in, to_ui_out) = triple_buffer(&vec![0u8; self.size]);
        let (mut to_audio_in, to_audio_out) = triple_buffer(&vec![0u8; self.size]);
        let (mut to_video_in, to_video_out) = triple_buffer(&vec![0u8; self.size]);
        spawn(move || loop {
            if from_ui_out.updated() {
                trace!("UI->audio bytecode update");
                let latest_ui_bytecode = from_ui_out.read();
                self.bytecode.copy_from_slice(latest_ui_bytecode.as_slice());
            }
            self.vm.run(&mut self.bytecode, &mut NoopBackend, true);
            to_ui_in.input_buffer().copy_from_slice(&self.bytecode);
            to_ui_in.publish();
            to_audio_in.input_buffer().copy_from_slice(&self.bytecode);
            to_audio_in.publish();
            to_video_in.input_buffer().copy_from_slice(&self.bytecode);
            to_video_in.publish();
        });

        BytecodeComms {
            bc_in: from_ui_in,
            ui_out: to_ui_out,
            audio_out: to_audio_out,
            video_out: to_video_out,
        }
    }
}
