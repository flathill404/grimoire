use nih_plug::prelude::*;
use std::sync::Arc;

mod constants;
mod dsp;
mod parameters;

use constants::*;
use dsp::DelayLine;
use parameters::DelayParams;

const MAX_DELAY_MS: f32 = 2000.0;

struct CantripDelay {
    params: Arc<DelayParams>,
    delay_lines: [DelayLine; 2],
    sample_rate: f32,
}

impl Default for CantripDelay {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayParams::default()),
            delay_lines: [
                DelayLine::new(MAX_DELAY_MS, 44100.0),
                DelayLine::new(MAX_DELAY_MS, 44100.0),
            ],
            sample_rate: 44100.0,
        }
    }
}

impl Plugin for CantripDelay {
    const NAME: &'static str = NAME;
    const VENDOR: &'static str = VENDOR;
    const URL: &'static str = URL;
    const EMAIL: &'static str = EMAIL;
    const VERSION: &'static str = VERSION;

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
        for delay_line in &mut self.delay_lines {
            delay_line.set_sample_rate(buffer_config.sample_rate, MAX_DELAY_MS);
        }
        true
    }

    fn reset(&mut self) {
        for delay_line in &mut self.delay_lines {
            delay_line.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let delay_time = self.params.delay_time.smoothed.next();
            let feedback = self.params.feedback.smoothed.next() / 100.0;
            let mix = self.params.mix.smoothed.next() / 100.0;

            for (channel_idx, sample) in channel_samples.iter_mut().enumerate() {
                let dry = *sample;
                let wet = self.delay_lines[channel_idx].process(dry, delay_time, feedback);

                let mut output = dry * (1.0 - mix) + wet * mix;

                if output.abs() < 1e-15 {
                    output = 0.0;
                }

                *sample = output;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for CantripDelay {
    const CLAP_ID: &'static str = CLAP_ID;
    const CLAP_DESCRIPTION: Option<&'static str> = CLAP_DESCRIPTION;
    const CLAP_MANUAL_URL: Option<&'static str> = CLAP_MANUAL_URL;
    const CLAP_SUPPORT_URL: Option<&'static str> = CLAP_SUPPORT_URL;
    const CLAP_FEATURES: &'static [ClapFeature] = CLAP_FEATURES;
}

impl Vst3Plugin for CantripDelay {
    const VST3_CLASS_ID: [u8; 16] = VST3_CLASS_ID;
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = VST3_SUBCATEGORIES;
}

nih_export_clap!(CantripDelay);
nih_export_vst3!(CantripDelay);
