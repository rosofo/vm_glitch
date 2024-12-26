use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;

use super::{
    timer::{Timer, TimerEvent},
    VmData, VmEvent,
};
use dasp::ring_buffer::Fixed;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use nih_plug_vizia::vizia::vg::Color;
use triple_buffer::Output;

type Buf = Arc<Mutex<Output<Vec<u8>>>>;
pub struct AnalyzerView<
    L1: Lens<Target = Buf>,
    L2: Lens<Target = (Arc<AtomicUsize>, Arc<AtomicUsize>)>,
> {
    bytecode: L1,
    counters: L2,
}

#[derive(Lens)]
struct Tracking {
    pc: dasp::ring_buffer::Fixed<[usize; 32]>,
}
impl Model for Tracking {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, _| match event {
            TrackingEvent::PC(pc) => {
                self.pc.push(*pc);
            }
        });
    }
}
enum TrackingEvent {
    PC(usize),
}

impl<L1: Lens<Target = Buf>, L2: Lens<Target = (Arc<AtomicUsize>, Arc<AtomicUsize>)>>
    AnalyzerView<L1, L2>
{
    pub fn new(cx: &mut Context, bytecode: L1, counters: L2) -> Handle<Self> {
        Timer {
            elapsed: Default::default(),
        }
        .build(cx);
        Tracking {
            pc: Fixed::from([0; 32]),
        }
        .build(cx);
        Self { bytecode, counters }
            .build(cx, |cx| {
                let pc = counters.get(cx).0.clone();
                cx.spawn(move |cx| loop {
                    let before = Instant::now();
                    std::thread::sleep(Duration::from_millis(100));
                    let after = Instant::now();
                    cx.emit(TimerEvent::Tick(after - before)).unwrap();
                    cx.emit(TrackingEvent::PC(
                        pc.load(std::sync::atomic::Ordering::Relaxed),
                    ))
                    .unwrap();
                });
            })
            .bind(Timer::elapsed, |mut handle, _| handle.needs_redraw())
    }
}

impl<L1: Lens<Target = Buf>, L2: Lens<Target = (Arc<AtomicUsize>, Arc<AtomicUsize>)>> View
    for AnalyzerView<L1, L2>
{
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let bytecode_ref = self.bytecode.get(cx);

        let pcs = Tracking::pc.get(cx);

        let mut guard = bytecode_ref.lock().unwrap();
        let bytecode = guard.read();

        canvas.clear_rect(
            bounds.x as u32,
            bounds.y as u32,
            600,
            20,
            Color::rgb(250, 250, 200),
        );
        for (i, pc) in pcs.iter().enumerate() {
            let x = bounds.x + (*pc as f32 / bytecode.len() as f32) * 600.0;
            canvas.clear_rect(
                x as u32,
                bounds.y as u32 + 1,
                15,
                19,
                Color::rgb(20, 20, 100 + (i as f32 / 32.0) as u8),
            );
        }

        let bit_scale = 8.0;
        let byte_scale = 1.5 * bit_scale;
        let coords = (0..bytecode.len()).map(|i| {
            let x = i % 32;
            let y = i / 32;
            (
                x as f32 * 2.0 * (byte_scale + 0.5) + bounds.x,
                y as f32 * 2.0 * (byte_scale) + bounds.y + 20.0,
                bytecode[i],
            )
        });
        for (byte_idx, (x, y, byte)) in coords.enumerate() {
            for i in 0..8 {
                let x = x + bit_scale * 1.25 * ((i % 4) as f32);
                let y = y + bit_scale * ((i / 4) as f32);
                let bit = (byte >> i) & 1;
                let mut color = if bit == 1 {
                    Color::rgb(252, 109, 171)
                } else {
                    Color::rgb(92, 65, 93)
                };
                let mut path = vg::Path::new();
                path.rect(x, y, bit_scale, bit_scale);
                let mut paint = vg::Paint::default();
                paint.set_color(color);
                canvas.fill_path(&path, &paint);
            }
        }
    }
}
