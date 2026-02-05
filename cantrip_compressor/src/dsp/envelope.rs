/// Envelope follower with separate attack and release times.
///
/// Uses a simple one-pole filter for smooth envelope tracking.
#[derive(Clone, Copy, Debug)]
pub struct EnvelopeFollower {
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Default for EnvelopeFollower {
    fn default() -> Self {
        Self {
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        }
    }
}

impl EnvelopeFollower {
    /// Reset the envelope state.
    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    /// Update attack and release coefficients based on time constants.
    ///
    /// # Arguments
    /// * `attack_ms` - Attack time in milliseconds
    /// * `release_ms` - Release time in milliseconds
    /// * `sample_rate` - Sample rate in Hz
    pub fn set_times(&mut self, attack_ms: f32, release_ms: f32, sample_rate: f32) {
        // Convert ms to coefficient: exp(-1 / (time_s * sample_rate))
        // This gives the time constant for a one-pole filter
        self.attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        self.release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
    }

    /// Process a single sample and return the current envelope level.
    ///
    /// Uses peak detection with separate attack/release smoothing.
    pub fn process(&mut self, input: f32) -> f32 {
        let input_abs = input.abs();

        let coeff = if input_abs > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        // One-pole smoothing filter
        self.envelope = input_abs + coeff * (self.envelope - input_abs);

        // Anti-denormal
        if self.envelope < 1e-15 {
            self.envelope = 0.0;
        }

        self.envelope
    }
}
