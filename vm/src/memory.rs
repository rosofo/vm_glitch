use std::ops::Index;

use byte::*;
use byte_slice_cast::*;
use ctx::NATIVE;
use dasp::*;
use eyre::bail;
use slice::{ToFrameSlice, ToFrameSliceMut};

/// A unified buffer with instructions and audio samples
///
/// ```
///           instructions                   audio               
///     ┌──────────────────────────┐  ┌──────────────────────┐   
///     │                          │  │                      │   
///   ┌─┴──────────────────────────┴──┴──────────────────────┴─┐
///   │ ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌~~~~─┐┌─────┐┌────┐│
///   │ │  ►  ││  0  ││  ◄  ││  ▼  ││   ~~~~~~~~~~~     ~~  ~~~│
///   │ │  ►  ││  ▲  ││  ◄  ││  ▼  ││ ~~~~~~     ~~~~~~~~~~~~ ││
///   │ └─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─~~──┘└────┘│
///   └────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug)]
pub struct Memory {
    pub buffer: Vec<u8>,
    program_len: usize,
}

pub enum Area {
    Bytecode,
    Sample,
}

pub struct Samples<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Iterator for Samples<'_> {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        self.bytes.read_with(&mut self.offset, NATIVE).ok()
    }
}

impl Memory {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn new(program_size: usize, audio_size: usize) -> eyre::Result<Self> {
        // f32_size * channels == 4 * 2
        let len = program_size + audio_size * 8;
        if len % 4 != 0 {
            bail!("Buffer length for given sizes must be divisible by 4")
        }
        Ok(Self {
            buffer: vec![0; len],
            program_len: program_size,
        })
    }
    pub fn area_type(&self, index: usize) -> Area {
        if index < self.program_len {
            Area::Bytecode
        } else {
            Area::Sample
        }
    }

    /// Read memory as byte, ignoring layout.
    ///
    /// This will treat bytecode as f32 samples if indexing into bytecode area
    pub fn get_as_byte(&self, index: usize) -> Option<u8> {
        let mut offset = index;
        self.buffer.read_with(&mut offset, NATIVE).ok()
    }

    /// Read memory as sample, ignoring layout.
    ///
    /// This will treat bytecode as f32 samples if indexing into bytecode area
    pub fn get_as_sample(&self, index: usize) -> Option<f32> {
        let mut offset = index;
        self.buffer.read_with(&mut offset, NATIVE).ok()
    }

    /// Iterate over the 'sample' area of the buffer
    pub fn iter_samples(&self) -> Samples {
        Samples {
            bytes: &self.buffer,
            offset: self.program_len,
        }
    }

    /// Slice into the program and sample memory, with samples cast to f32
    pub fn slices(&self) -> (&[u8], &[f32]) {
        let (bytes, sample_bytes) = self.buffer.split_at(self.program_len);
        (bytes, sample_bytes.as_slice_of().unwrap())
    }
    /// Mutably slice into the program and sample memory, with samples cast to f32
    pub fn slices_mut(&mut self) -> (&mut [u8], &mut [f32]) {
        let (bytes, sample_bytes) = self.buffer.split_at_mut(self.program_len);
        (bytes, sample_bytes.as_mut_slice_of().unwrap())
    }

    /// Treat the entire buffer as a slice of samples
    pub fn as_sample_slice(&self) -> &[f32] {
        self.buffer.as_slice_of().unwrap()
    }
    /// Treat the entire buffer as a mutable slice of samples
    pub fn as_mut_sample_slice(&mut self) -> &mut [f32] {
        self.buffer.as_mut_slice_of().unwrap()
    }

    /// Treat the entire buffer as a slice of frames
    pub fn as_frames(&self) -> &[[f32; 2]] {
        self.buffer[self.program_len..]
            .as_slice_of::<f32>()
            .unwrap()
            .to_frame_slice()
            .unwrap()
    }
    /// Treat the entire buffer as a mutable slice of frames
    pub fn as_mut_frames(&mut self) -> &mut [[f32; 2]] {
        self.buffer[self.program_len..]
            .as_mut_slice_of::<f32>()
            .unwrap()
            .to_frame_slice_mut()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_memory()(prog_len in 32..512usize, audio_len in 32..512usize) -> eyre::Result<Memory> {
            Memory::new(prog_len, audio_len)
        }
    }

    proptest! {
        #[test]
        fn test_slicing(memory in arb_memory(), i in any::<prop::sample::Index>()) {
            prop_assume!(memory.is_ok());
            let memory = memory.unwrap();
            memory.get_as_sample(i.index(memory.len()) / 4).unwrap();
        }
    }
}
