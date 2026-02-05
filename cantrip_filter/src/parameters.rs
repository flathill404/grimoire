use nih_plug::prelude::*;

use crate::dsp::coefficients::{BiquadCoefficients, FilterContext};

#[derive(Params)]
pub struct CantripFilterParams {
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,

    #[id = "freq"]
    pub frequency: FloatParam,

    #[id = "q"]
    pub resonance: FloatParam,

    /// Filter gain for Peaking EQ, Low Shelf, and High Shelf (in dB)
    #[id = "filter_gain"]
    pub filter_gain: FloatParam,

    /// Output gain
    #[id = "gain"]
    pub gain: FloatParam,
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum FilterType {
    // === Basic Filters (12dB/oct) ===
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
    #[name = "Notch"]
    Notch,
    #[name = "All Pass"]
    AllPass,

    // === Gentle Slope (6dB/oct) ===
    #[name = "Low Pass 6dB"]
    LowPass6dB,
    #[name = "High Pass 6dB"]
    HighPass6dB,

    // === EQ Types ===
    #[name = "Peaking EQ"]
    Peaking,
    #[name = "Low Shelf"]
    LowShelf,
    #[name = "High Shelf"]
    HighShelf,
    #[name = "Tilt"]
    Tilt,

    // === Crossover (Linkwitz-Riley) ===
    #[name = "LR Low Pass"]
    LinkwitzRileyLP,
    #[name = "LR High Pass"]
    LinkwitzRileyHP,

    // === Butterworth (maximally flat) ===
    #[name = "Butterworth LP"]
    ButterworthLP,
    #[name = "Butterworth HP"]
    ButterworthHP,

    // === Band Pass Variations ===
    #[name = "Band Pass 0dB"]
    BandPass0dB,

    // === Character / Creative ===
    #[name = "Warmth"]
    Warmth,
    #[name = "Brightness"]
    Brightness,
    #[name = "Presence"]
    Presence,
    #[name = "Air"]
    Air,
    #[name = "Sub Bass"]
    SubBass,
    #[name = "Vocal"]
    Vocal,

    // === Utility ===
    #[name = "DC Block"]
    DCBlock,
    #[name = "Unity"]
    Unity,
}

impl FilterType {
    /// Compute biquad coefficients for this filter type.
    pub fn compute_coefficients(
        self,
        freq: f32,
        q: f32,
        gain_db: f32,
        sample_rate: f32,
    ) -> BiquadCoefficients {
        let ctx = FilterContext::new(freq, q, gain_db, sample_rate);

        match self {
            // Basic filters
            Self::LowPass => Self::lowpass(&ctx),
            Self::HighPass => Self::highpass(&ctx),
            Self::BandPass => Self::bandpass(&ctx),
            Self::Notch => Self::notch(&ctx),
            Self::AllPass => Self::allpass(&ctx),

            // Gentle slope (6dB/oct)
            Self::LowPass6dB => Self::lowpass_6db(&ctx),
            Self::HighPass6dB => Self::highpass_6db(&ctx),

            // EQ types
            Self::Peaking => Self::peaking(&ctx),
            Self::LowShelf => Self::low_shelf(&ctx),
            Self::HighShelf => Self::high_shelf(&ctx),
            Self::Tilt => Self::tilt(&ctx, gain_db),

            // Crossover (Linkwitz-Riley)
            Self::LinkwitzRileyLP => Self::lowpass_with_q(&ctx, 0.5),
            Self::LinkwitzRileyHP => Self::highpass_with_q(&ctx, 0.5),

            // Butterworth
            Self::ButterworthLP => Self::lowpass_with_q(&ctx, std::f32::consts::FRAC_1_SQRT_2),
            Self::ButterworthHP => Self::highpass_with_q(&ctx, std::f32::consts::FRAC_1_SQRT_2),

            // Band pass variations
            Self::BandPass0dB => Self::bandpass_0db(&ctx, q),

            // Character filters
            Self::Warmth => Self::shelf_character(&ctx, gain_db, ShelfType::Low, 0.6),
            Self::Brightness => Self::shelf_character(&ctx, gain_db, ShelfType::High, 0.7),
            Self::Air => Self::shelf_character(&ctx, gain_db, ShelfType::High, 0.5),
            Self::SubBass => Self::shelf_character(&ctx, gain_db, ShelfType::Low, 0.8),
            Self::Presence => Self::peaking_character(&ctx, gain_db, 1.5),
            Self::Vocal => Self::peaking_character(&ctx, gain_db, 2.0),

            // Utility
            Self::DCBlock => Self::dc_block(sample_rate),
            Self::Unity => BiquadCoefficients::unity(),
        }
    }

    // ========================================
    // Basic Filters
    // ========================================

    fn lowpass(ctx: &FilterContext) -> BiquadCoefficients {
        let b1 = 1.0 - ctx.cos_w0;
        let b0 = b1 / 2.0;
        BiquadCoefficients::from_raw(
            b0,
            b1,
            b0,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    fn highpass(ctx: &FilterContext) -> BiquadCoefficients {
        let b1 = -(1.0 + ctx.cos_w0);
        let b0 = (1.0 + ctx.cos_w0) / 2.0;
        BiquadCoefficients::from_raw(
            b0,
            b1,
            b0,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    fn lowpass_with_q(ctx: &FilterContext, q: f32) -> BiquadCoefficients {
        let alpha = ctx.alpha_with_q(q);
        let b1 = 1.0 - ctx.cos_w0;
        let b0 = b1 / 2.0;
        BiquadCoefficients::from_raw(b0, b1, b0, 1.0 + alpha, -2.0 * ctx.cos_w0, 1.0 - alpha)
    }

    fn highpass_with_q(ctx: &FilterContext, q: f32) -> BiquadCoefficients {
        let alpha = ctx.alpha_with_q(q);
        let b1 = -(1.0 + ctx.cos_w0);
        let b0 = (1.0 + ctx.cos_w0) / 2.0;
        BiquadCoefficients::from_raw(b0, b1, b0, 1.0 + alpha, -2.0 * ctx.cos_w0, 1.0 - alpha)
    }

    fn bandpass(ctx: &FilterContext) -> BiquadCoefficients {
        BiquadCoefficients::from_raw(
            ctx.alpha,
            0.0,
            -ctx.alpha,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    fn bandpass_0db(ctx: &FilterContext, q: f32) -> BiquadCoefficients {
        BiquadCoefficients::from_raw(
            q * ctx.alpha,
            0.0,
            -q * ctx.alpha,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    fn notch(ctx: &FilterContext) -> BiquadCoefficients {
        BiquadCoefficients::from_raw(
            1.0,
            -2.0 * ctx.cos_w0,
            1.0,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    fn allpass(ctx: &FilterContext) -> BiquadCoefficients {
        BiquadCoefficients::from_raw(
            1.0 - ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 + ctx.alpha,
            1.0 + ctx.alpha,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha,
        )
    }

    // ========================================
    // 6dB/oct (1-pole approximation)
    // ========================================

    fn lowpass_6db(ctx: &FilterContext) -> BiquadCoefficients {
        let k = ctx.w0.tan() / 2.0;
        let norm = 1.0 / (1.0 + k);
        let b0 = k * norm;
        BiquadCoefficients::from_raw(b0, b0, 0.0, 1.0, (k - 1.0) * norm, 0.0)
    }

    fn highpass_6db(ctx: &FilterContext) -> BiquadCoefficients {
        let k = ctx.w0.tan() / 2.0;
        let norm = 1.0 / (1.0 + k);
        BiquadCoefficients::from_raw(norm, -norm, 0.0, 1.0, (k - 1.0) * norm, 0.0)
    }

    // ========================================
    // EQ Types
    // ========================================

    fn peaking(ctx: &FilterContext) -> BiquadCoefficients {
        BiquadCoefficients::from_raw(
            1.0 + ctx.alpha * ctx.a,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha * ctx.a,
            1.0 + ctx.alpha / ctx.a,
            -2.0 * ctx.cos_w0,
            1.0 - ctx.alpha / ctx.a,
        )
    }

    fn low_shelf(ctx: &FilterContext) -> BiquadCoefficients {
        Self::shelf_coefficients(ctx, ShelfType::Low)
    }

    fn high_shelf(ctx: &FilterContext) -> BiquadCoefficients {
        Self::shelf_coefficients(ctx, ShelfType::High)
    }

    fn tilt(ctx: &FilterContext, gain_db: f32) -> BiquadCoefficients {
        let tilt_a = 10.0f32.powf(gain_db / 20.0);
        let sqrt_a = tilt_a.sqrt();
        BiquadCoefficients::from_raw(
            sqrt_a * (sqrt_a + ctx.alpha / sqrt_a),
            -2.0 * sqrt_a * ctx.cos_w0,
            sqrt_a * (sqrt_a - ctx.alpha / sqrt_a),
            sqrt_a + ctx.alpha * sqrt_a,
            -2.0 * sqrt_a * ctx.cos_w0,
            sqrt_a - ctx.alpha * sqrt_a,
        )
    }

    // ========================================
    // Shelf Helpers
    // ========================================

    fn shelf_coefficients(ctx: &FilterContext, shelf_type: ShelfType) -> BiquadCoefficients {
        let two_sqrt_a_alpha = 2.0 * ctx.a.sqrt() * ctx.alpha;

        match shelf_type {
            ShelfType::Low => BiquadCoefficients::from_raw(
                ctx.a * ((ctx.a + 1.0) - (ctx.a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha),
                2.0 * ctx.a * ((ctx.a - 1.0) - (ctx.a + 1.0) * ctx.cos_w0),
                ctx.a * ((ctx.a + 1.0) - (ctx.a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha),
                (ctx.a + 1.0) + (ctx.a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha,
                -2.0 * ((ctx.a - 1.0) + (ctx.a + 1.0) * ctx.cos_w0),
                (ctx.a + 1.0) + (ctx.a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha,
            ),
            ShelfType::High => BiquadCoefficients::from_raw(
                ctx.a * ((ctx.a + 1.0) + (ctx.a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha),
                -2.0 * ctx.a * ((ctx.a - 1.0) + (ctx.a + 1.0) * ctx.cos_w0),
                ctx.a * ((ctx.a + 1.0) + (ctx.a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha),
                (ctx.a + 1.0) - (ctx.a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha,
                2.0 * ((ctx.a - 1.0) - (ctx.a + 1.0) * ctx.cos_w0),
                (ctx.a + 1.0) - (ctx.a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha,
            ),
        }
    }

    // ========================================
    // Character Filters
    // ========================================

    fn shelf_character(
        ctx: &FilterContext,
        gain_db: f32,
        shelf_type: ShelfType,
        q: f32,
    ) -> BiquadCoefficients {
        let a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
        let alpha = ctx.alpha_with_q(q);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        match shelf_type {
            ShelfType::Low => BiquadCoefficients::from_raw(
                a * ((a + 1.0) - (a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha),
                2.0 * a * ((a - 1.0) - (a + 1.0) * ctx.cos_w0),
                a * ((a + 1.0) - (a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha),
                (a + 1.0) + (a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha,
                -2.0 * ((a - 1.0) + (a + 1.0) * ctx.cos_w0),
                (a + 1.0) + (a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha,
            ),
            ShelfType::High => BiquadCoefficients::from_raw(
                a * ((a + 1.0) + (a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * ctx.cos_w0),
                a * ((a + 1.0) + (a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha),
                (a + 1.0) - (a - 1.0) * ctx.cos_w0 + two_sqrt_a_alpha,
                2.0 * ((a - 1.0) - (a + 1.0) * ctx.cos_w0),
                (a + 1.0) - (a - 1.0) * ctx.cos_w0 - two_sqrt_a_alpha,
            ),
        }
    }

    fn peaking_character(ctx: &FilterContext, gain_db: f32, q: f32) -> BiquadCoefficients {
        let a = 10.0f32.powf(gain_db.max(3.0) / 40.0);
        let alpha = ctx.alpha_with_q(q);
        BiquadCoefficients::from_raw(
            1.0 + alpha * a,
            -2.0 * ctx.cos_w0,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * ctx.cos_w0,
            1.0 - alpha / a,
        )
    }

    // ========================================
    // Utility
    // ========================================

    fn dc_block(sample_rate: f32) -> BiquadCoefficients {
        use std::f32::consts::PI;
        let dc_w0 = 2.0 * PI * 20.0 / sample_rate;
        let dc_cos = dc_w0.cos();
        let dc_alpha = dc_w0.sin() / (2.0 * 0.707);
        let b0 = (1.0 + dc_cos) / 2.0;
        BiquadCoefficients::from_raw(
            b0,
            -(1.0 + dc_cos),
            b0,
            1.0 + dc_alpha,
            -2.0 * dc_cos,
            1.0 - dc_alpha,
        )
    }
}

#[derive(Clone, Copy)]
enum ShelfType {
    Low,
    High,
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
                    factor: FloatRange::skew_factor(-2.0),
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
            filter_gain: FloatParam::new(
                "Filter Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
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
