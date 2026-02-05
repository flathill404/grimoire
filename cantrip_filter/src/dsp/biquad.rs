use crate::dsp::coefficients::BiquadCoefficients;
use crate::parameters::FilterType;

/// A biquad (two-pole, two-zero) digital filter.
///
/// Implements the standard Direct Form 1 difference equation:
/// y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
#[derive(Clone, Copy, Debug, Default)]
pub struct Biquad {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State (delay line)
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the filter state (delay line) to zero.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Update filter coefficients for the given filter type and parameters.
    pub fn update(
        &mut self,
        filter_type: FilterType,
        freq: f32,
        q: f32,
        gain_db: f32,
        sample_rate: f32,
    ) {
        let coeffs = filter_type.compute_coefficients(freq, q, gain_db, sample_rate);
        self.set_coefficients(coeffs);
    }

    /// Directly set the filter coefficients.
    pub fn set_coefficients(&mut self, coeffs: BiquadCoefficients) {
        self.b0 = coeffs.b0;
        self.b1 = coeffs.b1;
        self.b2 = coeffs.b2;
        self.a1 = coeffs.a1;
        self.a2 = coeffs.a2;
    }

    /// Process a single sample through the filter.
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Anti-denormal: flush very small values to zero.
        // This prevents CPU spikes when the signal decays to subnormal values.
        let output = if output.abs() < 1e-15 { 0.0 } else { output };

        // Update delay line
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}
