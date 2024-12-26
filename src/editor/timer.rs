use std::time::Instant;

use drawille::Canvas;
use nih_plug_vizia::vizia::prelude::*;

#[derive(Lens)]
pub struct Timer {
    pub elapsed: Duration,
}
#[derive(Debug)]
pub enum TimerEvent {
    Tick(Duration),
}

impl Model for Timer {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, _| {
            if let TimerEvent::Tick(delta) = event {
                self.elapsed += *delta;
            }
        });
    }
}
