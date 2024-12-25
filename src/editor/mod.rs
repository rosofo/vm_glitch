mod analyzer;
mod logo;
use generate::generate;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use crate::VmGlitchParams;
use analyzer::AnalyzerView;
use lang::*;
use logo::Logo;
use tracing::{instrument, trace};
use triple_buffer::{Input, Output};

#[derive(Lens)]
struct Data {
    params: Arc<VmGlitchParams>,
    from_vm_buffer: Arc<Mutex<Output<Vec<u8>>>>,
    to_vm_buffer: Arc<Mutex<Input<Vec<u8>>>>,
    errs: String,
    counters: (Arc<AtomicUsize>, Arc<AtomicUsize>),
}

impl Model for Data {
    #[instrument(skip(self, cx, event))]
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::Edit(s) => {
                let mut guard = self.params.code.lock().unwrap();
                *guard = s.clone();
                let str = guard.clone();
                let parsed = lang::parse::parse(&guard);
                match parsed {
                    Ok(gtch) => {
                        self.errs = "".to_string();
                        let bytecode = lang::compile::compile(
                            &gtch,
                            self.from_vm_buffer
                                .lock()
                                .unwrap()
                                .peek_output_buffer()
                                .len(),
                        );
                        let Ok(bytecode) = bytecode else {
                            let errs = bytecode.unwrap_err();
                            println!("{}", errs);
                            self.errs = format!("{:#?}", errs);
                            return;
                        };
                        {
                            let mut guard = self.to_vm_buffer.lock().unwrap();
                            trace!("->audio: publish bytecode");
                            guard.write(bytecode);
                        }
                    }
                    Err(errs) => {
                        self.errs = format!("{:#?}", errs);
                    }
                };
            }
            AppEvent::Gen => {
                cx.emit(AppEvent::Edit(generate()));
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
    Gen,
}

pub(crate) fn create(
    params: Arc<VmGlitchParams>,
    editor_state: Arc<ViziaState>,
    from_vm_buffer: Output<Vec<u8>>,
    to_vm_buffer: Input<Vec<u8>>,
    counters: (Arc<AtomicUsize>, Arc<AtomicUsize>),
) -> Option<Box<dyn Editor>> {
    // need these to be Arc<Mutex<...>> only for the UI thread, there's no blocking from the audio thread.
    let from_vm_buffer = Arc::new(Mutex::new(from_vm_buffer));
    let to_vm_buffer = Arc::new(Mutex::new(to_vm_buffer));
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);
        register_doto_font(cx);

        Data {
            params: params.clone(),
            from_vm_buffer: from_vm_buffer.clone(),
            to_vm_buffer: to_vm_buffer.clone(),
            errs: "".to_string(),
            counters: counters.clone(),
        }
        .build(cx);

        ZStack::new(cx, |cx| {
            Logo::new(cx);
            VStack::new(cx, |cx| {
                nih_plug_vizia::vizia::views::Label::new(cx, "VM Glitch")
                    .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                    .font_weight(FontWeightKeyword::Thin)
                    .font_size(30.0)
                    .height(Pixels(100.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Pixels(0.0));

                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |cx| cx.emit(AppEvent::Gen),
                        |cx| nih_plug_vizia::vizia::views::Label::new(cx, "Generate"),
                    );
                    Textbox::new(cx, Data::params.map(|p| p.code.lock().unwrap().clone()))
                        .on_edit(|cx, s| cx.emit(AppEvent::Edit(s)))
                        .min_width(Pixels(300.0));
                });

                nih_plug_vizia::vizia::views::Label::new(cx, Data::errs).width(Pixels(300.0));

                AnalyzerView::new(cx, Data::from_vm_buffer, Data::counters)
                    .width(Pixels(500.0))
                    .child_space(Stretch(1.0));
            })
            .row_between(Pixels(0.0))
            .width(Stretch(1.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));
        })
        .background_color(nih_plug_vizia::vizia::style::Color::rgb(247, 255, 247))
        .width(Percentage(100.0))
        .height(Percentage(100.0));

        ResizeHandle::new(cx);
        cx.emit(GuiContextEvent::Resize);
    })
}

fn register_doto_font(cx: &mut Context) {
    cx.add_font_mem(include_bytes!("../../assets/Doto.ttf"));
}
