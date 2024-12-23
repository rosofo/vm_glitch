use std::ops::Index;

use byte::*;
use byte_slice_cast::*;
use ctx::NATIVE;
use dasp::*;

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
pub struct Memory {
    pub buffer: Vec<u8>,
    pub program_len: usize,
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
    pub fn area_type(&self, index: usize) -> Area {
        if index < self.program_len {
            Area::Bytecode
        } else {
            Area::Sample
        }
    }
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
}
