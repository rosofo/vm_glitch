use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Offsets {
    #[bits(16)]
    bytecode_start: usize,
    #[bits(16)]
    data_start: usize,
}

pub struct Registers {
    // Read-only registers
    offsets: Offsets,
    pc: u32,
    // r1: u32,
    // r2: u32,
    // r3: u32,
    // r4: u32
}

pub struct Processor {
    registers: Registers,
}

impl Processor {
    pub fn process(mut buffer: &[u8]) {
        todo!("weeeeeee :3")
    }
}
