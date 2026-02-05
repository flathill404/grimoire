use nih_plug::prelude::*;

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
