use std::sync::Arc;

use chute::{
    mpmc::{Queue, Reader, Writer},
    LendingReader,
};

/// An interface to the message bus
pub struct Messages {
    writer: Writer<Message>,
    reader: Reader<Message>,
}

impl Messages {
    pub fn new(bus: &Arc<Queue<Message>>) -> Self {
        Self {
            writer: bus.writer(),
            reader: bus.reader(),
        }
    }
    pub fn send(&mut self, message: Message) {
        self.writer.push(message);
    }
    pub fn recv(&mut self) -> Option<&Message> {
        self.reader.next()
    }
}

#[derive(Clone, Debug)]
pub struct Message;
