mod analyzer;
mod delay_buffer;
mod editor;
mod logo;
use delay_buffer::DelayBuffer;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    vec,
};
use triple_buffer::{triple_buffer, Input, Output};
use vm::Vm;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct VmGlitch {
    params: Arc<VmGlitchParams>,
    vm: Vm,
    to_ui_buffer: (Input<Vec<u8>>, Option<Output<Vec<u8>>>),
    from_ui_buffer: (Option<Input<Vec<u8>>>, Output<Vec<u8>>),
    dirty: Arc<AtomicBool>,
    delay_buffer: DelayBuffer,
}

#[derive(Params)]
pub struct VmGlitchParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,

    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[persist = "code"]
    pub code: Arc<Mutex<String>>,
}

impl Default for VmGlitch {
    fn default() -> Self {
        let vm = Vm::default();
        let to_ui = triple_buffer(&vec![0; 512]);
        let from_ui = triple_buffer(&vec![0; 512]);
        Self {
            params: Arc::new(VmGlitchParams::default()),
            vm: Vm::default(),
            dirty: Arc::new(AtomicBool::new(false)),
            to_ui_buffer: (to_ui.0, Some(to_ui.1)),
            from_ui_buffer: (Some(from_ui.0), from_ui.1),
            // ??? use audio buffer * 2, which is dynamic ofc
            delay_buffer: DelayBuffer::new(1024),
        }
    }
}

impl Default for VmGlitchParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            code: Default::default(),

            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let samples = buffer.samples();
        // TODO decide whether to just get updates from the UI thread periodically, e.g. every X calls.
        // Would be less complex, though it would mean the bytecode gets reset regardless of whether the user edited it.
        if let Ok(true) =
            self.dirty
                .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
        {
            let updated_bytecode = self.from_ui_buffer.1.read();
            // copy the user's new bytecode without publishing back to the UI thread yet.
            self.to_ui_buffer
                .0
                .input_buffer()
                .copy_from_slice(updated_bytecode.as_slice());
        }

        self.delay_buffer.copy_to_back();
        self.delay_buffer.ingest_audio(buffer.as_slice());

        self.vm.run(
            self.to_ui_buffer.0.input_buffer(), // stage updates for the UI thread without publishing yet
            self.delay_buffer.as_mut_slice(),
            samples,
        );

        self.delay_buffer.write_to_audio(buffer.as_slice());

        self.to_ui_buffer.0.publish();

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.to_ui_buffer.1.take().unwrap(),
            self.from_ui_buffer.0.take().unwrap(),
            self.dirty.clone(),
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
