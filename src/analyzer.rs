use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;

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

impl<L1: Lens<Target = Buf>, L2: Lens<Target = (Arc<AtomicUsize>, Arc<AtomicUsize>)>>
    AnalyzerView<L1, L2>
{
    pub fn new(cx: &mut Context, bytecode: L1, counters: L2) -> Handle<Self> {
        Self { bytecode, counters }
            .build(cx, |cx| {})
            // Redraw when lensed data changes
            .bind(bytecode, |mut handle, _| handle.needs_redraw())
    }
}

impl<L1: Lens<Target = Buf>, L2: Lens<Target = (Arc<AtomicUsize>, Arc<AtomicUsize>)>> View
    for AnalyzerView<L1, L2>
{
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let bytecode_ref = self.bytecode.get(cx);

        let (pc, buf_index) = self.counters.get(cx);
        let pc = pc.load(std::sync::atomic::Ordering::Relaxed);
        let buf_index = buf_index.load(std::sync::atomic::Ordering::Relaxed);

        let mut guard = bytecode_ref.lock().unwrap();
        let bytecode = guard.read();

        let bit_scale = 8.0;
        let byte_scale = 1.5 * bit_scale;
        let coords = (0..bytecode.len()).map(|i| {
            let x = i % 32;
            let y = i / 32;
            (
                x as f32 * 2.0 * (byte_scale + 0.5) + bounds.x,
                y as f32 * 2.0 * (byte_scale) + bounds.y,
                bytecode[i],
            )
        });
        for (byte_idx, (x, y, byte)) in coords.enumerate() {
            if byte_idx == pc {
                let mut path = vg::Path::new();
                path.rect(x - 2.0, y - 2.0, byte_scale + 2.0, byte_scale + 2.0);
                let mut paint = vg::Paint::default();
                paint.set_color(Color::rgb(0, 0, 255));
                canvas.fill_path(&path, &paint);
            }
            for i in 0..8 {
                let x = x + bit_scale * ((i % 4) as f32);
                let y = y + bit_scale * ((i / 4) as f32);
                let bit = (byte >> i) & 1;
                let mut color = if bit == 1 {
                    Color::rgb(252, 109, 171)
                } else {
                    Color::rgb(92, 65, 93)
                };
                if byte_idx == pc {
                    color.b += 0.5;
                }
                let mut path = vg::Path::new();
                path.rect(x, y, bit_scale, bit_scale);
                let mut paint = vg::Paint::default();
                paint.set_color(color);
                canvas.fill_path(&path, &paint);
            }
        }
    }
}
