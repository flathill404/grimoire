use nih_plug::prelude::*;

#[derive(Params)]
pub struct CantripCompressorParams {
    /// Threshold in dB - level above which compression begins
    #[id = "threshold"]
    pub threshold: FloatParam,

    /// Ratio - compression ratio (e.g., 4.0 means 4:1)
    #[id = "ratio"]
    pub ratio: FloatParam,

    /// Attack time in milliseconds
    #[id = "attack"]
    pub attack: FloatParam,

    /// Release time in milliseconds
    #[id = "release"]
    pub release: FloatParam,

    /// Knee width in dB (0 = hard knee)
    #[id = "knee"]
    pub knee: FloatParam,

    /// Makeup gain in dB
    #[id = "makeup"]
    pub makeup: FloatParam,

    /// Mix (dry/wet) - 0% = dry, 100% = wet
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for CantripCompressorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                -20.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),

            ratio: FloatParam::new(
                "Ratio",
                4.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(":1")
            .with_step_size(0.1),

            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 100.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" ms")
            .with_step_size(0.1),

            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" ms")
            .with_step_size(1.0),

            knee: FloatParam::new(
                "Knee",
                6.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),

            makeup: FloatParam::new(
                "Makeup",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 30.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),

            mix: FloatParam::new(
                "Mix",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(1.0),
        }
    }
}
