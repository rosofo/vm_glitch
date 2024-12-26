use std::{fmt::Debug, sync::Arc, thread::JoinHandle};

use chute::mpmc::Queue;

use crate::bus::{Message, Messages};

/// A runner for VM modules
///
/// # Example
///
/// ```no_run
/// Runtime::default().spawn(VideoMod.default())
/// ```
pub struct Runtime {
    /// Handles for all the spawned module threads
    handles: Vec<JoinHandle<()>>,
    /// A 'broadcast'/'multicast' message bus allowing all modules to communicate
    bus: Arc<Queue<Message>>,
}

/// A piece of functionality running on its own thread
pub trait Module: Debug {
    fn spawn(self, messages: Messages) -> JoinHandle<()>;
}

impl Default for Runtime {
    fn default() -> Self {
        let bus = Queue::new();
        Self {
            handles: vec![],
            bus,
        }
    }
}

impl Runtime {
    /// Spawn a module, giving it a reference to the message bus
    pub fn spawn<M: Module + 'static>(&mut self, module: M) -> &mut Self {
        let messages = self.messages();
        self.handles.push(module.spawn(messages));
        self
    }

    /// Get an interface to send and receive messages.
    ///
    /// This is likely to be needed for the audio module, as its thread is spawned separately.
    pub fn messages(&self) -> Messages {
        Messages::new(&self.bus)
    }
}
