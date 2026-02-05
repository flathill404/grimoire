use nih_plug::prelude::*;
use std::sync::Arc;

mod constants;
mod dsp;
mod parameters;

use constants::*;
use dsp::biquad::Biquad;
use parameters::CantripFilterParams;

struct CantripFilter {
    params: Arc<CantripFilterParams>,
    // Stereo filter state
    filters: [Biquad; 2],
    sample_rate: f32,
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

impl Plugin for CantripFilter {
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
        // Update filter coefficients once per block
        let filter_type = self.params.filter_type.value();
        let freq = self.params.frequency.value();
        let q = self.params.resonance.value();

        for filter in &mut self.filters {
            filter.update(filter_type, freq, q, 0.0, self.sample_rate);
        }

        // Process sample by sample
        // iter_samples() iterates per-sample, giving access to all channels for each sample
        for mut channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for (channel_idx, sample) in channel_samples.iter_mut().enumerate() {
                if channel_idx < self.filters.len() {
                    *sample = self.filters[channel_idx].process(*sample) * gain;
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for CantripFilter {
    const CLAP_ID: &'static str = CLAP_ID;
    const CLAP_DESCRIPTION: Option<&'static str> = CLAP_DESCRIPTION;
    const CLAP_MANUAL_URL: Option<&'static str> = CLAP_MANUAL_URL;
    const CLAP_SUPPORT_URL: Option<&'static str> = CLAP_SUPPORT_URL;
    const CLAP_FEATURES: &'static [ClapFeature] = CLAP_FEATURES;
}

impl Vst3Plugin for CantripFilter {
    const VST3_CLASS_ID: [u8; 16] = VST3_CLASS_ID;
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = VST3_SUBCATEGORIES;
}

nih_export_clap!(CantripFilter);
nih_export_vst3!(CantripFilter);

#[cfg(test)]
mod tests {
    use super::dsp::biquad::Biquad;
    use super::parameters::FilterType;

    #[test]
    fn test_biquad_lowpass_dc_gain() {
        let mut filter = Biquad::new();
        // Set lowpass at 1kHz, Q=0.707, Sample rate 44.1kHz
        filter.update(FilterType::LowPass, 1000.0, 0.707, 0.0, 44100.0);
        
        // Feed DC (1.0) for a while and check if it stabilizes near 1.0 (0dB gain at DC for LP)
        let mut output = 0.0;
        for _ in 0..1000 {
            output = filter.process(1.0);
        }
        
        assert!((output - 1.0).abs() < 1e-4, "LowPass DC gain should be close to 1.0, got {}", output);
    }

    #[test]
    fn test_biquad_highpass_dc_rejection() {
        let mut filter = Biquad::new();
        // Set highpass at 1kHz
        filter.update(FilterType::HighPass, 1000.0, 0.707, 0.0, 44100.0);

        // Feed DC (1.0) - should be rejected
        let mut output = 0.0;
        for _ in 0..1000 {
            output = filter.process(1.0);
        }

        assert!(output.abs() < 1e-4, "HighPass DC output should be close to 0.0, got {}", output);
    }
}
