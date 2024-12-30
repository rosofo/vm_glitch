mod delay_buffer;
mod editor;
mod threads;
#[cfg(feature = "tracing")]
mod trace;
use delay_buffer::DelayBuffer;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{
    sync::{Arc, Mutex},
    thread::{self, spawn},
    time::Duration,
    vec,
};
use threads::{BytecodeComms, BytecodeThread};
use tracing::{instrument, trace};
use triple_buffer::{triple_buffer, Input, Output};
use vm::backend::Backend;
use vm::interpret::Vm;

pub type BytecodeUpdates = Vec<u8>;

#[derive(derive_more::Debug)]
pub struct VmGlitch {
    #[debug(ignore)]
    params: Arc<VmGlitchParams>,
    vm: Vm,
    delay_buffer: DelayBuffer,
    bytecode: Option<Output<Vec<u8>>>,
    bytecode_rate: Arc<AtomicF32>,
}

#[derive(Params)]
pub struct VmGlitchParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "bytecode_rate"]
    pub bytecode_rate: FloatParam,

    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[persist = "code"]
    pub code: Arc<Mutex<String>>,
}

impl Default for VmGlitch {
    fn default() -> Self {
        let to_ui = triple_buffer(&vec![0; 512]);
        let from_ui = triple_buffer(&vec![0; 512]);

        #[cfg(feature = "tracing")]
        trace::setup();

        Self {
            params: Arc::new(VmGlitchParams::default()),
            vm: Vm::default(),
            delay_buffer: DelayBuffer::new(8192),
            bytecode: None,
            bytecode_rate: Arc::new(AtomicF32::new(0.5)),
        }
    }
}

impl Default for VmGlitchParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            code: Default::default(),

            bytecode_rate: FloatParam::new(
                "Bytecode Rate",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_unit(" secs"),
        }
    }
}

impl Plugin for VmGlitch {
    const NAME: &'static str = "VM Glitch";
    const VENDOR: &'static str = "Robin Forbes";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "rsforbes0@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    #[instrument(skip(self, buffer, _aux, _context))]
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.delay_buffer.ingest_audio(buffer);

        if let Some(bytecode) = self.bytecode.as_mut() {
            bytecode.update();
            // run vm on audio without bytecode self-mod
            self.vm.run(
                bytecode.output_buffer(),
                &mut self.delay_buffer.buffer,
                false,
            );
        }

        self.delay_buffer.write_to_audio(buffer);

        #[cfg(feature = "tracing")]
        tracy_client::Client::running().unwrap().frame_mark();

        self.bytecode_rate.store(
            self.params.bytecode_rate.value(),
            std::sync::atomic::Ordering::Relaxed,
        );

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let (tx, rx) = crossbeam_channel::bounded(100);
        let rate = Arc::clone(&self.bytecode_rate);
        spawn(move || loop {
            tx.send(threads::Message::ModBytecode).unwrap();
            thread::sleep(Duration::from_secs_f32(
                rate.load(std::sync::atomic::Ordering::Relaxed),
            ));
        });
        let BytecodeComms {
            bc_in,
            ui_out,
            audio_out,
            video_out,
        } = BytecodeThread::new(512, rx).spawn();
        self.bytecode = Some(audio_out);
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            ui_out,
            bc_in,
            self.vm.ui_counters.clone(),
        )
    }
}

impl ClapPlugin for VmGlitch {
    const CLAP_ID: &'static str = "com.sandiskette.vm-glitch";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Manipulate audio with a fool-vulnerable DSL");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for VmGlitch {
    const VST3_CLASS_ID: [u8; 16] = *b"sdkvmglitch12345";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(VmGlitch);
nih_export_vst3!(VmGlitch);
