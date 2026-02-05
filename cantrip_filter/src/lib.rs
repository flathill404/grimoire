use nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

struct CantripFilter {
    params: Arc<CantripFilterParams>,
    // Stereo filter state
    filters: [Biquad; 2],
    sample_rate: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Biquad {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn update(&mut self, filter_type: FilterType, freq: f32, q: f32, _gain_db: f32, sample_rate: f32) {
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        // let a = 10.0f32.powf(gain_db / 40.0); // For peaking/shelving - unused for now

        let (b0, b1, b2, a0, a1, a2) = match filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize
        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        let mut output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Anti-denormal / Flush-to-zero
        // This prevents CPU spikes and potential noise when the signal decays to very small values.
        if output.abs() < 1e-11 {
            output = 0.0;
        }

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

#[derive(Params)]
struct CantripFilterParams {
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,

    #[id = "freq"]
    pub frequency: FloatParam,

    #[id = "q"]
    pub resonance: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum FilterType {
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
}

impl Default for CantripFilter {
    fn default() -> Self {
        Self {
            params: Arc::new(CantripFilterParams::default()),
            filters: [Biquad::new(); 2],
            sample_rate: 44100.0,
        }
    }
}

impl Default for CantripFilterParams {
    fn default() -> Self {
        Self {
            filter_type: EnumParam::new("Type", FilterType::LowPass),
            frequency: FloatParam::new(
                "Frequency",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            resonance: FloatParam::new(
                "Resonance",
                0.707,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 10.0,
                    factor: FloatRange::skew_factor(0.5),
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for CantripFilter {
    const NAME: &'static str = "Cantrip Filter";
    const VENDOR: &'static str = "flathill404";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "38638577+flathill404@users.noreply.github.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        for filter in &mut self.filters {
            filter.reset();
        }
        true
    }

    fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Prepare gain smoothing for the block
        // Since parameters are shared, we can calculate the gain curve once.
        // However, `smoothed.next()` advances the state. If we call it for each channel loop,
        // it would advance twice as fast if we are naive.
        // NIH-plug's `Buffer` stores channels separately.
        // Correct approach: Collect gain values into a temporary buffer for the block size,
        // then reuse it for each channel.
        let num_samples = buffer.samples();
        let mut gain_values = vec![0.0; num_samples];
        for i in 0..num_samples {
            gain_values[i] = self.params.gain.smoothed.next();
        }

        for (channel_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            if channel_idx >= self.filters.len() {
                break;
            }
            let filter = &mut self.filters[channel_idx];

            let filter_type = self.params.filter_type.value();
            let freq = self.params.frequency.value();
            let q = self.params.resonance.value();
            
            // Note: Filter coefficients are updated once per block. 
            // For smoother filter modulation, we'd need per-sample or small-block updates.
            filter.update(filter_type, freq, q, 0.0, self.sample_rate);

            for (sample_idx, sample) in channel_samples.iter_mut().enumerate() {
                *sample = filter.process(*sample) * gain_values[sample_idx];
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for CantripFilter {
    const CLAP_ID: &'static str = "com.flathill404.grimoire.cantrip_filter";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple Biquad Filter");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Filter, ClapFeature::Stereo];
}

impl Vst3Plugin for CantripFilter {
    const VST3_CLASS_ID: [u8; 16] = *b"hCfVdKlz609eczKi";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_clap!(CantripFilter);
nih_export_vst3!(CantripFilter);
