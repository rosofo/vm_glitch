use std::sync::Arc;
use std::sync::Mutex;

use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use nih_plug_vizia::vizia::vg::Color;
use nih_plug_vizia::vizia::vg::Quad;
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

        let coords = (0..bytecode.len()).map(|i| {
            let x = i % 16;
            let y = i / 16;
            (
                x as f32 * 30.0 + bounds.x,
                y as f32 * 30.0 + bounds.y,
                bytecode[i],
            )
        });
        for (x, y, byte) in coords {
            let mut path = vg::Path::new();
            path.rect(x, y, 10.0, 10.0);
            let mut paint = vg::Paint::default();
            paint.set_color(Color::rgb(byte, byte, byte));
            canvas.fill_path(&path, &paint);
        }
    }
}
