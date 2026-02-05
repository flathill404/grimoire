use nih_plug::prelude::*;

#[derive(Params)]
pub struct DelayParams {
    #[id = "time"]
    pub delay_time: FloatParam,

    #[id = "feedback"]
    pub feedback: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            delay_time: FloatParam::new(
                "Delay Time",
                250.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" ms")
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            feedback: FloatParam::new(
                "Feedback",
                30.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit(" %")
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            mix: FloatParam::new("Mix", 50.0, FloatRange::Linear { min: 0.0, max: 100.0 })
                .with_unit(" %")
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}
