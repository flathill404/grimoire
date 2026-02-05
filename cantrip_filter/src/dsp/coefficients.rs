/// Normalized biquad filter coefficients.
///
/// These are the coefficients after normalization by a0.
/// Transfer function: H(z) = (b0 + b1*z^-1 + b2*z^-2) / (1 + a1*z^-1 + a2*z^-2)
#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadCoefficients {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoefficients {
    /// Create unity (pass-through) coefficients.
    pub fn unity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Create coefficients from raw (unnormalized) values.
    /// Automatically normalizes by a0.
    pub fn from_raw(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        let inv_a0 = 1.0 / a0;
        Self {
            b0: b0 * inv_a0,
            b1: b1 * inv_a0,
            b2: b2 * inv_a0,
            a1: a1 * inv_a0,
            a2: a2 * inv_a0,
        }
    }
}

/// Pre-computed intermediate values for biquad coefficient calculation.
/// Avoids redundant trigonometric computations.
pub struct FilterContext {
    pub w0: f32,
    pub cos_w0: f32,
    pub sin_w0: f32,
    pub alpha: f32,
    pub a: f32, // Gain factor for peaking/shelving
}

impl FilterContext {
    pub fn new(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> Self {
        use std::f32::consts::PI;

        // Clamp frequency to valid range
        let freq = freq.clamp(1.0, sample_rate * 0.499);

        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        let a = 10.0f32.powf(gain_db / 40.0);

        Self {
            w0,
            cos_w0,
            sin_w0,
            alpha,
            a,
        }
    }

    /// Create alpha with a custom Q value.
    pub fn alpha_with_q(&self, q: f32) -> f32 {
        self.sin_w0 / (2.0 * q)
    }
}
