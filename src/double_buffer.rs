use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

#[derive(Debug, Default)]
pub struct DoubleBuffer {
    buffers: [Mutex<Vec<u8>>; 2],
    write_index: AtomicBool,
    length: usize,
}

impl DoubleBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buffers: [Mutex::new(vec![0; size]), Mutex::new(vec![0; size])],
            write_index: AtomicBool::new(false),
            length: size,
        }
    }

    pub fn write_buffer(&self) -> impl std::ops::DerefMut<Target = Vec<u8>> + '_ {
        // This should never fail since writer has exclusive access
        self.buffers[self.write_index.load(Ordering::Acquire) as usize]
            .try_lock()
            .expect("Write buffer locked unexpectedly")
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn read_buffer(&self) -> impl std::ops::Deref<Target = Vec<u8>> + '_ {
        // This should never fail since reader has exclusive access
        self.buffers[!self.write_index.load(Ordering::Acquire) as usize]
            .try_lock()
            .expect("Read buffer locked unexpectedly")
    }

    pub fn swap(&self) {
        self.write_index.fetch_xor(true, Ordering::AcqRel);
    }
}
