use std::ops::Index;

use dasp::*;
use variantly::Variantly;

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

#[derive(derive_more::From, Variantly, Clone, PartialEq, Debug)]
pub enum Value {
    Bytecode(u8),
    Sample(f32),
}

impl Value {
    pub fn from_sample_bytes(bytes: &[u8]) -> Option<Self> {
        Some(
            f32::from_le_bytes([
                *bytes.first()?,
                *bytes.get(1)?,
                *bytes.get(2)?,
                *bytes.get(3)?,
            ])
            .into(),
        )
    }
}

#[derive(Variantly, derive_more::From)]
pub enum ValueBytes {
    Byte(u8),
    Sample([u8; 4]),
}

impl From<f32> for ValueBytes {
    fn from(value: f32) -> Self {
        Self::Sample(value.to_le_bytes())
    }
}

impl From<Value> for ValueBytes {
    fn from(value: Value) -> Self {
        match value {
            Value::Bytecode(byte) => byte.into(),
            Value::Sample(sample) => sample.into(),
        }
    }
}

pub enum MemoryType {
    Bytecode,
    Sample,
}

#[derive(derive_more::Deref)]
pub struct SampleIndex(usize);

impl Memory {
    pub fn type_of(&self, index: usize) -> MemoryType {
        if index < self.program_len {
            MemoryType::Bytecode
        } else {
            MemoryType::Sample
        }
    }
    fn get(&self, index: usize) -> Option<Value> {
        match self.type_of(index) {
            MemoryType::Bytecode => {
                let byte = self.buffer.get(index)?;
                Some(Value::Bytecode(*byte))
            }
            MemoryType::Sample => {
                let bytes = self.buffer.get(index..index + 4)?;
                Value::from_sample_bytes(bytes)
            }
        }
    }

    fn set(&mut self, index: usize, value: impl Into<ValueBytes>) {
        let bytes: ValueBytes = value.into();
        match bytes {
            ValueBytes::Byte(byte) => {
                *self.buffer.get_mut(index).unwrap() = byte;
            }
            ValueBytes::Sample(bytes) => {
                self.buffer
                    .get_mut(index..index + bytes.len())
                    .unwrap()
                    .copy_from_slice(&bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_get_and_set_with_existing_is_identity(i in 0..10usize, buf in prop::collection::vec(prop::bits::u8::ANY, 10)) {
            let mut memory = Memory {buffer: buf, program_len: 4};
            if let Some(value) = memory.get(i) {
                let orig = value.clone();
                memory.set(i, value);
                let value = memory.get(i).unwrap();
                prop_assert_eq!(orig, value);
            }
        }
    }
}
