// #[bitfield(u32)]
// pub struct Metadata {
//     #[bits(4)]
//     bytecode_length: usize,
// }

pub struct Registers {
    pc: u32,
    metadata: u32,
    // r1: u32,
    // r2: u32,
    // r3: u32,
    // r4: u32
}

pub struct Processor {
    registers: Registers,
    buffer: [u8],
}

impl Processor {
    pub fn process(mut buffer: &[u8]) {
        todo!("weeeeeee :3")
    }
}
