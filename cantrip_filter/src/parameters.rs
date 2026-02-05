use nih_plug::prelude::*;

#[derive(Params)]
pub struct CantripFilterParams {
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,

    #[id = "freq"]
    pub frequency: FloatParam,

    #[id = "q"]
    pub resonance: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum FilterType {
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
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
