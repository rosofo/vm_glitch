use std::thread::{spawn, JoinHandle};

use crate::{bus::Messages, runtime::Module};

#[derive(Debug)]
pub struct AudioMod {}

impl Module for AudioMod {
    fn spawn(self, messages: Messages) -> JoinHandle<()> {
        spawn(|| {})
    }
}
