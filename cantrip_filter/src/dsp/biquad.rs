use std::f32::consts::PI;
use crate::parameters::FilterType;

#[derive(Clone, Copy, Debug, Default)]
pub struct Biquad {
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn update(&mut self, filter_type: FilterType, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        // Clamp frequency to valid range (avoiding instability near Nyquist)
        let freq = freq.clamp(1.0, sample_rate * 0.499);

        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        // A for peaking and shelving EQ
        let a = 10.0f32.powf(gain_db / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match filter_type {
            // ========================================
            // Basic Filters (12dB/oct, 2-pole)
            // ========================================
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
                // Constant skirt gain, peak gain = Q
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::AllPass => {
                let b0 = 1.0 - alpha;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 + alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Gentle Slope (6dB/oct, 1-pole approximation)
            // ========================================
            FilterType::LowPass6dB => {
                // One-pole lowpass approximated as biquad
                let k = w0.tan() / 2.0;
                let norm = 1.0 / (1.0 + k);
                let b0 = k * norm;
                let b1 = k * norm;
                let b2 = 0.0;
                let a0 = 1.0;
                let a1 = (k - 1.0) * norm;
                let a2 = 0.0;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass6dB => {
                // One-pole highpass approximated as biquad
                let k = w0.tan() / 2.0;
                let norm = 1.0 / (1.0 + k);
                let b0 = norm;
                let b1 = -norm;
                let b2 = 0.0;
                let a0 = 1.0;
                let a1 = (k - 1.0) * norm;
                let a2 = 0.0;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // EQ Types
            // ========================================
            FilterType::Peaking => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::LowShelf => {
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighShelf => {
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Tilt => {
                // Tilt EQ: simultaneously boosts high frequencies and cuts low frequencies (or vice versa)
                // Uses gain_db: positive = bright tilt, negative = warm tilt
                // Frequency sets the pivot point
                let tilt_a = 10.0f32.powf(gain_db / 20.0);
                let sqrt_a = tilt_a.sqrt();
                let b0 = sqrt_a * (sqrt_a + alpha / sqrt_a);
                let b1 = -2.0 * sqrt_a * cos_w0;
                let b2 = sqrt_a * (sqrt_a - alpha / sqrt_a);
                let a0 = sqrt_a + alpha * sqrt_a;
                let a1 = -2.0 * sqrt_a * cos_w0;
                let a2 = sqrt_a - alpha * sqrt_a;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Crossover (Linkwitz-Riley)
            // ========================================
            FilterType::LinkwitzRileyLP => {
                // Linkwitz-Riley 2nd order (12dB/oct with -6dB at crossover)
                // Q = 0.5 for Linkwitz-Riley
                let lr_alpha = sin_w0 / (2.0 * 0.5);
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + lr_alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - lr_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::LinkwitzRileyHP => {
                // Linkwitz-Riley 2nd order highpass
                let lr_alpha = sin_w0 / (2.0 * 0.5);
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + lr_alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - lr_alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Butterworth (maximally flat passband)
            // ========================================
            FilterType::ButterworthLP => {
                // Butterworth Q = 1/sqrt(2) ≈ 0.7071
                let bw_alpha = sin_w0 / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + bw_alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - bw_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::ButterworthHP => {
                let bw_alpha = sin_w0 / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + bw_alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - bw_alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Band Pass Variations
            // ========================================
            FilterType::BandPass0dB => {
                // Constant 0dB peak gain (normalized)
                let b0 = q * alpha;
                let b1 = 0.0;
                let b2 = -q * alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Character / Creative Filters
            // ========================================
            FilterType::Warmth => {
                // Low-mid boost around the frequency parameter
                // Uses a gentle low shelf with some resonance
                let warmth_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let warmth_alpha = sin_w0 / (2.0 * 0.6);
                let two_sqrt_a_alpha = 2.0 * warmth_a.sqrt() * warmth_alpha;
                let b0 = warmth_a * ((warmth_a + 1.0) - (warmth_a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = 2.0 * warmth_a * ((warmth_a - 1.0) - (warmth_a + 1.0) * cos_w0);
                let b2 = warmth_a * ((warmth_a + 1.0) - (warmth_a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (warmth_a + 1.0) + (warmth_a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = -2.0 * ((warmth_a - 1.0) + (warmth_a + 1.0) * cos_w0);
                let a2 = (warmth_a + 1.0) + (warmth_a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Brightness => {
                // High frequency boost/enhancement using high shelf
                let bright_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let bright_alpha = sin_w0 / (2.0 * 0.7);
                let two_sqrt_a_alpha = 2.0 * bright_a.sqrt() * bright_alpha;
                let b0 = bright_a * ((bright_a + 1.0) + (bright_a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = -2.0 * bright_a * ((bright_a - 1.0) + (bright_a + 1.0) * cos_w0);
                let b2 = bright_a * ((bright_a + 1.0) + (bright_a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (bright_a + 1.0) - (bright_a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = 2.0 * ((bright_a - 1.0) - (bright_a + 1.0) * cos_w0);
                let a2 = (bright_a + 1.0) - (bright_a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Presence => {
                // Presence boost in the 2-5kHz range (upper mids)
                // Uses peaking EQ characteristics
                let pres_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let pres_alpha = sin_w0 / (2.0 * 1.5); // Moderate Q
                let b0 = 1.0 + pres_alpha * pres_a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - pres_alpha * pres_a;
                let a0 = 1.0 + pres_alpha / pres_a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - pres_alpha / pres_a;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Air => {
                // "Air" - very high frequency boost (10kHz+)
                // High shelf with gentle slope
                let air_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let air_alpha = sin_w0 / (2.0 * 0.5);
                let two_sqrt_a_alpha = 2.0 * air_a.sqrt() * air_alpha;
                let b0 = air_a * ((air_a + 1.0) + (air_a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = -2.0 * air_a * ((air_a - 1.0) + (air_a + 1.0) * cos_w0);
                let b2 = air_a * ((air_a + 1.0) + (air_a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (air_a + 1.0) - (air_a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = 2.0 * ((air_a - 1.0) - (air_a + 1.0) * cos_w0);
                let a2 = (air_a + 1.0) - (air_a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::SubBass => {
                // Sub bass enhancement - low shelf optimized for very low frequencies
                let sub_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let sub_alpha = sin_w0 / (2.0 * 0.8);
                let two_sqrt_a_alpha = 2.0 * sub_a.sqrt() * sub_alpha;
                let b0 = sub_a * ((sub_a + 1.0) - (sub_a - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = 2.0 * sub_a * ((sub_a - 1.0) - (sub_a + 1.0) * cos_w0);
                let b2 = sub_a * ((sub_a + 1.0) - (sub_a - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (sub_a + 1.0) + (sub_a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = -2.0 * ((sub_a - 1.0) + (sub_a + 1.0) * cos_w0);
                let a2 = (sub_a + 1.0) + (sub_a - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Vocal => {
                // Vocal presence - peaking EQ optimized for voice intelligibility
                // Center around 2-4kHz with moderate Q
                let vocal_a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
                let vocal_alpha = sin_w0 / (2.0 * 2.0); // Tighter Q for vocal clarity
                let b0 = 1.0 + vocal_alpha * vocal_a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - vocal_alpha * vocal_a;
                let a0 = 1.0 + vocal_alpha / vocal_a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - vocal_alpha / vocal_a;
                (b0, b1, b2, a0, a1, a2)
            }

            // ========================================
            // Utility
            // ========================================
            FilterType::DCBlock => {
                // DC blocking filter (high pass at very low frequency)
                // Fixed at ~20Hz regardless of freq parameter
                let dc_w0 = 2.0 * PI * 20.0 / sample_rate;
                let dc_cos = dc_w0.cos();
                let dc_alpha = dc_w0.sin() / (2.0 * 0.707);
                let b0 = (1.0 + dc_cos) / 2.0;
                let b1 = -(1.0 + dc_cos);
                let b2 = (1.0 + dc_cos) / 2.0;
                let a0 = 1.0 + dc_alpha;
                let a1 = -2.0 * dc_cos;
                let a2 = 1.0 - dc_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Unity => {
                // Pass-through (no filtering)
                let b0 = 1.0;
                let b1 = 0.0;
                let b2 = 0.0;
                let a0 = 1.0;
                let a1 = 0.0;
                let a2 = 0.0;
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

    pub fn process(&mut self, input: f32) -> f32 {
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
