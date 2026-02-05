use super::envelope::EnvelopeFollower;

/// Compressor gain computer and processor.
///
/// Handles the core compression logic: envelope detection, gain calculation,
/// and gain smoothing.
#[derive(Clone, Copy, Debug, Default)]
pub struct Compressor {
    envelope: EnvelopeFollower,
}

impl Compressor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the compressor state.
    pub fn reset(&mut self) {
        self.envelope.reset();
    }

    /// Update the envelope follower timing.
    pub fn set_times(&mut self, attack_ms: f32, release_ms: f32, sample_rate: f32) {
        self.envelope.set_times(attack_ms, release_ms, sample_rate);
    }

    /// Compute gain reduction in dB for a given input level.
    ///
    /// # Arguments
    /// * `input_db` - Input level in dB
    /// * `threshold_db` - Threshold in dB
    /// * `ratio` - Compression ratio (e.g., 4.0 for 4:1)
    /// * `knee_db` - Knee width in dB (0 = hard knee)
    ///
    /// # Returns
    /// Gain reduction in dB (negative value)
    fn compute_gain_reduction(
        input_db: f32,
        threshold_db: f32,
        ratio: f32,
        knee_db: f32,
    ) -> f32 {
        let half_knee = knee_db / 2.0;

        if knee_db > 0.0 && input_db > (threshold_db - half_knee) && input_db < (threshold_db + half_knee) {
            // Soft knee region
            let x = input_db - threshold_db + half_knee;
            let compression = (1.0 / ratio - 1.0) * x * x / (2.0 * knee_db);
            compression
        } else if input_db >= threshold_db + half_knee {
            // Above knee - full compression
            let excess = input_db - threshold_db;
            let compressed_excess = excess / ratio;
            compressed_excess - excess
        } else {
            // Below threshold - no compression
            0.0
        }
    }

    /// Process a stereo pair and return the gain to apply (linear).
    ///
    /// Uses the maximum of both channels for detection (linked stereo).
    pub fn process_stereo(
        &mut self,
        left: f32,
        right: f32,
        threshold_db: f32,
        ratio: f32,
        knee_db: f32,
    ) -> f32 {
        // Use max of both channels (linked stereo)
        let input = left.abs().max(right.abs());

        // Get smoothed envelope
        let envelope = self.envelope.process(input);

        // Convert to dB (with floor to avoid -inf)
        let input_db = if envelope > 1e-10 {
            20.0 * envelope.log10()
        } else {
            -100.0
        };

        // Compute gain reduction
        let gain_reduction_db = Self::compute_gain_reduction(input_db, threshold_db, ratio, knee_db);

        // Convert back to linear gain
        10.0f32.powf(gain_reduction_db / 20.0)
    }
}
