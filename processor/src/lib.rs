use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Offsets {
    #[bits(16)]
    data_start: u16,
    #[bits(16)]
    _padding: u16,
}

pub struct Registers {
    // Read-only registers
    offsets: Offsets,
    memory_len: u32,
    pc: u32,
    // r1: u32,
    // r2: u32,
    // r3: u32,
    // r4: u32
}

impl Registers {
    pub fn reset(&mut self) {
        self.pc = 0;
    }
}

pub struct VmProcessor {
    registers: Registers,
}

impl Processor for VmProcessor {
    fn reset(&mut self) {
        todo!()
    }

    fn run(&mut self, memory: & mut [u8]) {
        todo!()
    }
}

pub trait Processor {
    fn reset(&mut self);
    fn run(&mut self, memory: & mut [u8]);
}
