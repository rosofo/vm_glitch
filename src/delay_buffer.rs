use dasp::ring_buffer::Fixed;
use nih_plug::buffer::Buffer;

#[derive(Debug)]
pub struct DelayBuffer {
    pub buffer: Fixed<Vec<[f32; 2]>>,
}

impl DelayBuffer {
    pub fn new(len: usize) -> Self {
        Self {
            buffer: Fixed::from(vec![[0.0, 0.0]; len]),
        }
    }

    /// Push incoming samples to the back of the buffer
    pub fn ingest_audio(&mut self, audio: &mut Buffer) {
        #[cfg(feature = "tracing")]
        let _span = tracy_client::span!("delay buffer: Ingest new audio samples");
        let audio = audio.as_slice();
        for (left, right) in audio[0].iter().zip(audio[1].iter()) {
            self.buffer.push([*left, *right]);
        }
    }

    /// Copy front to the audio buffer
    pub fn write_to_audio(&self, audio: &mut Buffer) {
        #[cfg(feature = "tracing")]
        let _span = tracy_client::span!("delay buffer: Fill audio buffer with output");
        for (frame, mut chan_iter) in self.buffer.iter().zip(audio.iter_samples()) {
            *chan_iter.get_mut(0).unwrap() = frame[0];
            *chan_iter.get_mut(1).unwrap() = frame[1];
        }
    }
}
