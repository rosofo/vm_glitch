pub struct DelayBuffer {
    pub buffer: Vec<Vec<f32>>,
}

impl DelayBuffer {
    pub fn new(len: usize) -> Self {
        Self {
            buffer: vec![vec![0.0; len], vec![0.0; len]],
        }
    }
    // TODO maybe make private and call in [ingest_audio]
    /// Copy second half to first half in preparation for new samples
    pub fn copy_to_back(&mut self) {
        for chan in self.buffer.iter_mut() {
            let len = chan.len();
            chan.copy_within(len / 2..len, 0);
        }
    }
    /// Copy audio buffer to second half of self
    pub fn ingest_audio(&mut self, audio: &mut [&mut [f32]]) {
        for (self_chan, audio_chan) in self.buffer.iter_mut().zip(audio.iter()) {
            let self_len = self_chan.len();
            for (self_sample, audio_sample) in self_chan
                .iter_mut()
                .skip(self_len / 2)
                .zip(audio_chan.iter())
            {
                *self_sample = *audio_sample;
            }
        }
    }

    /// Copy first half (oldest) to audio buffer
    pub fn write_to_audio(&self, audio: &mut [&mut [f32]]) {
        for (self_chan, audio_chan) in self.buffer.iter().zip(audio.iter_mut()) {
            for (self_sample, audio_sample) in self_chan.iter().zip(audio_chan.iter_mut()) {
                *audio_sample = *self_sample;
            }
        }
    }

    /// Can't return `&mut [&mut [f32]]` like nih_plug does cause it requires unsafe nastiness
    pub fn as_mut_slice(&mut self) -> &mut [Vec<f32>] {
        &mut self.buffer
    }
}
