use std::{default, time::Instant};

use drawille::Canvas;
use nih_plug_vizia::vizia::prelude::*;

#[derive(Lens)]
pub struct LogoData {
    elapsed: Duration,
}
#[derive(Debug)]
enum LogoEvent {
    Tick(Duration),
}

impl Model for LogoData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, _| {
            if let LogoEvent::Tick(delta) = event {
                println!("adding to elapsed");
                self.elapsed += *delta;
            }
        });
    }
}

pub struct Logo {}

impl Logo {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            LogoData {
                elapsed: Default::default(),
            }
            .build(cx);
            cx.spawn(|cx| loop {
                let before = Instant::now();
                std::thread::sleep(Duration::from_millis(100));
                let after = Instant::now();
                cx.emit(LogoEvent::Tick(after - before)).unwrap();
            });
            Binding::new(cx, LogoData::elapsed, |cx, lens| {
                let text = {
                    let elapsed = lens.get(cx);
                    println!("elapsed: {:?}", elapsed);
                    let mut canvas = Canvas::new(30, 30);
                    let speed = 2.0;
                    for i in 0..5 {
                        let phase = (elapsed.as_secs_f32() * (0.1 + i as f32 * 0.1)).sin();
                        let tx = (((elapsed.as_secs_f32() - phase) * speed).cos() / 2.0) + 0.5;
                        let ty = (((elapsed.as_secs_f32() + phase) * speed).sin() / 2.0) + 0.5;
                        let x = 1.0 + tx * 28.0;
                        let y = 1.0 + ty * 28.0;
                        let char = ["v", "m", "g", "l", "i", "t", "c", "h"][(tx * 7.0) as usize];
                        canvas.text(x as u32, y as u32, 1, char);
                    }
                    canvas.frame()
                };
                Label::new(cx, &text).font_family(vec![FamilyOwned::Name("Doto".to_string())]);
            });
        })
    }
}

impl View for Logo {}
