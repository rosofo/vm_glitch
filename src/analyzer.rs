use std::sync::Arc;
use std::sync::Mutex;

use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use nih_plug_vizia::vizia::vg::Color;
use triple_buffer::Output;

type Buf = Arc<Mutex<Output<Vec<u8>>>>;
pub struct AnalyzerView<L: Lens<Target = Buf>> {
    bytecode: L,
}

impl<L: Lens<Target = Buf>> AnalyzerView<L> {
    pub fn new(cx: &mut Context, bytecode: L) -> Handle<Self> {
        Self { bytecode }
            .build(cx, |cx| {
                Label::new(
                    cx,
                    bytecode.map(|buf| format!("{:?}", buf.lock().unwrap().read().len())),
                );
            })
            // Redraw when lensed data changes
            .bind(bytecode, |mut handle, _| handle.needs_redraw())
    }
}

impl<L: Lens<Target = Buf>> View for AnalyzerView<L> {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let bytecode_ref = self.bytecode.get(cx);
        let mut guard = bytecode_ref.lock().unwrap();
        let bytecode = guard.read();

        let bit_scale = 4.0;
        let byte_scale = 2.0 * bit_scale;
        let coords = (0..bytecode.len()).map(|i| {
            let x = i % 32;
            let y = i / 32;
            (
                x as f32 * 2.0 * (byte_scale + 1.0) + bounds.x,
                y as f32 * 2.0 * (byte_scale + 1.0) + bounds.y,
                bytecode[i],
            )
        });
        for (x, y, byte) in coords {
            for i in 0..8 {
                let x = x + bit_scale * ((i % 4) as f32);
                let y = y + bit_scale * ((i / 4) as f32);
                let bit = (byte >> i) & 1;
                let color = if bit == 1 {
                    Color::rgb(255, 255, 255)
                } else {
                    Color::rgb(0, 0, 0)
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
