use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct CantripGain {
    params: Arc<CantripGainParams>,
}

#[derive(Params)]
struct CantripGainParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,

    /// The pan parameter, range -1.0 to 1.0
    #[id = "pan"]
    pub pan: FloatParam,
}

impl Default for CantripGain {
    fn default() -> Self {
        Self {
            params: Arc::new(CantripGainParams::default()),
        }
    }
}

impl Default for CantripGainParams {
    fn default() -> Self {
        Self {
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

            pan: FloatParam::new(
                "Pan",
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(""), // No unit for pan, or maybe "%"? Usually just arbitrary.
        }
    }
}

impl Plugin for CantripGain {
    const NAME: &'static str = "Cantrip Gain";
    const VENDOR: &'static str = "flathill404";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "38638577+flathill404@users.noreply.github.com";

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
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let pan = self.params.pan.smoothed.next();

            // Equal power panning
            // pan range [-1.0, 1.0]
            // angle range [0, PI/2] -> (pan + 1.0) * PI / 4.0
            let pan_angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
            let pan_l = pan_angle.cos();
            let pan_r = pan_angle.sin();

            // We assume stereo output.
            // If input is mono, we might need to handle it, but AUDIO_IO_LAYOUTS says 2 channels.
            // buffer.iter_samples() yields iterators over channels.
            // But we need to process L and R differently.
            // iter_samples yields *frames*? No, let's check docs or usage.
            // nih-plug docs: buffer.iter_samples() returns an iterator that yields `&mut [f32]`.
            // The slice length is the number of channels.
            // So `channel_samples` is effectively `&mut [f32]` where len == num_channels.
            
            let samples = channel_samples.into_iter();
            let mut samples_iter = samples;

            if let (Some(l), Some(r)) = (samples_iter.next(), samples_iter.next()) {
                let in_l = *l;
                let in_r = *r;

                *l = in_l * gain * pan_l;
                *r = in_r * gain * pan_r;
            }
            // If there are more channels, we ignore them or process them? 
            // For now, let's assume stereo.
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for CantripGain {
    const CLAP_ID: &'static str = "com.flathill404.grimoire.cantrip_gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("simple gain");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for CantripGain {
    const VST3_CLASS_ID: [u8; 16] = *b"DEbWVppQipb8Cj7e";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(CantripGain);
nih_export_vst3!(CantripGain);
