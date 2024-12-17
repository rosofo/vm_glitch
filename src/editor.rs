use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::fmt::LowerHex;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::VmGlitchParams;
use lang::*;
use vm::Vm;

#[derive(Lens)]
struct Data {
    params: Arc<VmGlitchParams>,
    audio_vm: Arc<Mutex<Vm>>,
    code: String,
    bytecode_size: usize,
    errs: String,
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::Edit(s) => {
                self.code = s.clone();
                match lang::parse::parse(&self.code) {
                    Ok(gtch) => {
                        self.errs = "".to_string();
                        let bytecode = lang::assemble::assemble(gtch.iter(), self.bytecode_size);
                        let mut guard = self
                            .audio_vm
                            .lock()
                            .expect("failed to lock to push bytecode change");

                        guard.bytecode.copy_from_slice(&bytecode);
                    }
                    Err(errs) => {
                        self.errs = format!("{:#?}", errs);
                    }
                };
            }
        });
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (600, 400))
}

enum AppEvent {
    Edit(String),
}

pub(crate) fn create(
    params: Arc<VmGlitchParams>,
    editor_state: Arc<ViziaState>,
    vm: Arc<Mutex<Vm>>,
    bytecode_size: usize,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            audio_vm: vm.clone(),
            code: "".to_string(),
            bytecode_size,
            errs: "".to_string(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            nih_plug_vizia::vizia::views::Label::new(cx, "VM Glitch")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(30.0)
                .height(Pixels(100.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Textbox::new(cx, Data::code).on_edit(|cx, s| cx.emit(AppEvent::Edit(s)));
            nih_plug_vizia::vizia::views::Label::new(cx, Data::errs).width(Pixels(300.0));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));

        ResizeHandle::new(cx);
        cx.emit(GuiContextEvent::Resize);
    })
}
