use nih_plug::prelude::*;
use std::sync::Arc;

mod constants;
mod dsp;
mod parameters;

use constants::*;
use dsp::compressor::Compressor;
use parameters::CantripCompressorParams;

struct CantripCompressor {
    params: Arc<CantripCompressorParams>,
    compressor: Compressor,
    sample_rate: f32,
}

impl Default for CantripCompressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CantripCompressorParams::default()),
            compressor: Compressor::new(),
            sample_rate: 44100.0,
        }
    }
}

impl Plugin for CantripCompressor {
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
        self.compressor.reset();
        true
    }

    fn reset(&mut self) {
        self.compressor.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Get parameter values
        let threshold = self.params.threshold.value();
        let ratio = self.params.ratio.value();
        let attack = self.params.attack.value();
        let release = self.params.release.value();
        let knee = self.params.knee.value();
        let makeup_db = self.params.makeup.value();
        let mix = self.params.mix.value() / 100.0;

        // Update compressor timing
        self.compressor.set_times(attack, release, self.sample_rate);

        // Convert makeup gain to linear
        let makeup_gain = 10.0f32.powf(makeup_db / 20.0);

        // Process sample by sample
        for mut channel_samples in buffer.iter_samples() {
            // Get stereo samples
            let mut samples: [f32; 2] = [0.0; 2];
            for (i, sample) in channel_samples.iter_mut().enumerate() {
                if i < 2 {
                    samples[i] = *sample;
                }
            }

            // Compute gain reduction (linked stereo)
            let gain = self.compressor.process_stereo(
                samples[0],
                samples[1],
                threshold,
                ratio,
                knee,
            );

            // Apply gain with makeup and mix
            for (i, sample) in channel_samples.iter_mut().enumerate() {
                if i < 2 {
                    let dry = samples[i];
                    let wet = samples[i] * gain * makeup_gain;
                    *sample = dry * (1.0 - mix) + wet * mix;
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for CantripCompressor {
    const CLAP_ID: &'static str = CLAP_ID;
    const CLAP_DESCRIPTION: Option<&'static str> = CLAP_DESCRIPTION;
    const CLAP_MANUAL_URL: Option<&'static str> = CLAP_MANUAL_URL;
    const CLAP_SUPPORT_URL: Option<&'static str> = CLAP_SUPPORT_URL;
    const CLAP_FEATURES: &'static [ClapFeature] = CLAP_FEATURES;
}

impl Vst3Plugin for CantripCompressor {
    const VST3_CLASS_ID: [u8; 16] = VST3_CLASS_ID;
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = VST3_SUBCATEGORIES;
}

nih_export_clap!(CantripCompressor);
nih_export_vst3!(CantripCompressor);

#[cfg(test)]
mod tests {
    use super::dsp::compressor::Compressor;

    #[test]
    fn test_compressor_no_reduction_below_threshold() {
        let mut comp = Compressor::new();
        comp.set_times(10.0, 100.0, 44100.0);

        // Feed a quiet signal (well below -20dB threshold)
        let gain = comp.process_stereo(0.01, 0.01, -20.0, 4.0, 0.0);

        // Should be close to 1.0 (no reduction)
        assert!(
            (gain - 1.0).abs() < 0.01,
            "Expected gain ~1.0, got {}",
            gain
        );
    }

    #[test]
    fn test_compressor_reduces_above_threshold() {
        let mut comp = Compressor::new();
        comp.set_times(0.1, 100.0, 44100.0); // Very fast attack

        // Feed a loud signal repeatedly to build up envelope
        let mut gain = 1.0;
        for _ in 0..1000 {
            gain = comp.process_stereo(1.0, 1.0, -20.0, 4.0, 0.0);
        }

        // Should have significant reduction
        assert!(
            gain < 0.5,
            "Expected gain reduction, got {}",
            gain
        );
    }
}
