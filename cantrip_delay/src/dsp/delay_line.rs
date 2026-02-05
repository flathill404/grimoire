pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    sample_rate: f32,
}

impl DelayLine {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0).ceil() as usize + 1;
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32, max_delay_ms: f32) {
        self.sample_rate = sample_rate;
        let max_samples = (max_delay_ms * sample_rate / 1000.0).ceil() as usize + 1;
        self.buffer.resize(max_samples, 0.0);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    pub fn process(&mut self, input: f32, delay_ms: f32, feedback: f32) -> f32 {
        let delay_samples = (delay_ms * self.sample_rate / 1000.0) as usize;
        let delay_samples = delay_samples.min(self.buffer.len() - 1);

        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.buffer.len() - (delay_samples - self.write_pos)
        };

        let delayed = self.buffer[read_pos];

        self.buffer[self.write_pos] = input + delayed * feedback;

        self.write_pos += 1;
        if self.write_pos >= self.buffer.len() {
            self.write_pos = 0;
        }

        delayed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_basic() {
        let mut delay = DelayLine::new(100.0, 1000.0);

        delay.process(1.0, 10.0, 0.0);

        for _ in 0..9 {
            let output = delay.process(0.0, 10.0, 0.0);
            assert_eq!(output, 0.0);
        }

        let output = delay.process(0.0, 10.0, 0.0);
        assert_eq!(output, 1.0);
    }

    #[test]
    fn test_delay_line_feedback() {
        let mut delay = DelayLine::new(100.0, 1000.0);

        delay.process(1.0, 10.0, 0.5);

        for _ in 0..9 {
            delay.process(0.0, 10.0, 0.5);
        }

        let output = delay.process(0.0, 10.0, 0.5);
        assert_eq!(output, 1.0);

        for _ in 0..9 {
            delay.process(0.0, 10.0, 0.5);
        }

        let output = delay.process(0.0, 10.0, 0.5);
        assert!((output - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_delay_line_reset() {
        let mut delay = DelayLine::new(100.0, 1000.0);

        delay.process(1.0, 10.0, 0.5);
        delay.reset();

        for _ in 0..20 {
            let output = delay.process(0.0, 10.0, 0.5);
            assert_eq!(output, 0.0);
        }
    }
}
