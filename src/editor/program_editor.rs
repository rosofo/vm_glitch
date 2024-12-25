use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt},
    context::{Context, EmitContext},
    layout::Units::Pixels,
    modifiers::LayoutModifiers,
    view::{Handle, View},
    views::{Button, HStack, Textbox},
};

use super::{VmData, VmEvent};

pub struct ProgramEdit {}

impl ProgramEdit {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| cx.emit(VmEvent::Gen),
                    |cx| nih_plug_vizia::vizia::views::Label::new(cx, "Generate"),
                );
                Textbox::new(cx, VmData::params.map(|p| p.code.lock().unwrap().clone()))
                    .on_edit(|cx, s| cx.emit(VmEvent::Edit(s)))
                    .min_width(Pixels(300.0));
            });

            nih_plug_vizia::vizia::views::Label::new(cx, VmData::errs).width(Pixels(300.0));
        })
    }
}

impl View for ProgramEdit {}
