#[cfg(feature = "candle")]
pub mod silero_candle;

pub const SILERO_8KHZ_FRAME_SAMPLES: usize = 256;
pub const SILERO_16KHZ_FRAME_SAMPLES: usize = 512;

pub trait VadEngine: Send {
    fn speech_probability(&mut self, samples: &[f32], sample_rate: u32) -> Result<f32, ModelError>;
    fn reset(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct EnergyVad {
    scale: f32,
}

impl EnergyVad {
    pub fn new(scale: f32) -> Self {
        Self { scale }
    }
}

impl Default for EnergyVad {
    fn default() -> Self {
        Self { scale: 20.0 }
    }
}

impl VadEngine for EnergyVad {
    fn speech_probability(&mut self, samples: &[f32], sample_rate: u32) -> Result<f32, ModelError> {
        validate_silero_frame(samples, sample_rate)?;
        let rms = (samples.iter().map(|sample| sample * sample).sum::<f32>()
            / samples.len() as f32)
            .sqrt();
        Ok((rms * self.scale).clamp(0.0, 1.0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelError {
    UnsupportedSampleRate(u32),
    InvalidFrameSize {
        sample_rate: u32,
        expected: usize,
        actual: usize,
    },
    Model(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSampleRate(sample_rate) => {
                write!(formatter, "unsupported sample rate {sample_rate}")
            }
            Self::InvalidFrameSize {
                sample_rate,
                expected,
                actual,
            } => write!(
                formatter,
                "invalid Silero frame size for {sample_rate} Hz: expected {expected}, got {actual}"
            ),
            Self::Model(error) => write!(formatter, "model error: {error}"),
        }
    }
}

impl std::error::Error for ModelError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeechEvent {
    Started { sample_index: u64 },
    Ended { sample_index: u64 },
}

#[derive(Debug, Clone)]
pub struct SpeechSegmenter {
    threshold: f32,
    negative_threshold: f32,
    min_silence_samples: u64,
    speech_pad_samples: u64,
    triggered: bool,
    temp_end: Option<u64>,
    current_sample: u64,
}

impl SpeechSegmenter {
    pub fn new(sample_rate: u32, threshold: f32, min_silence_ms: u32, speech_pad_ms: u32) -> Self {
        Self::new_with_thresholds(
            sample_rate,
            threshold,
            (threshold - 0.15).max(0.01),
            min_silence_ms,
            speech_pad_ms,
        )
    }

    pub fn new_with_thresholds(
        sample_rate: u32,
        threshold: f32,
        negative_threshold: f32,
        min_silence_ms: u32,
        speech_pad_ms: u32,
    ) -> Self {
        Self {
            threshold,
            negative_threshold,
            min_silence_samples: sample_rate as u64 * min_silence_ms as u64 / 1_000,
            speech_pad_samples: sample_rate as u64 * speech_pad_ms as u64 / 1_000,
            triggered: false,
            temp_end: None,
            current_sample: 0,
        }
    }

    pub fn reset(&mut self) {
        self.triggered = false;
        self.temp_end = None;
    }

    pub fn observe(&mut self, probability: f32, frame_samples: usize) -> Option<SpeechEvent> {
        self.current_sample += frame_samples as u64;

        if probability >= self.threshold && self.temp_end.is_some() {
            self.temp_end = None;
        }

        if probability >= self.threshold && !self.triggered {
            self.triggered = true;
            let start = self
                .current_sample
                .saturating_sub(frame_samples as u64)
                .saturating_sub(self.speech_pad_samples);
            return Some(SpeechEvent::Started {
                sample_index: start,
            });
        }

        if probability < self.negative_threshold && self.triggered {
            let temp_end = *self.temp_end.get_or_insert(self.current_sample);
            if self.current_sample.saturating_sub(temp_end) >= self.min_silence_samples {
                self.temp_end = None;
                self.triggered = false;
                let end = temp_end
                    .saturating_sub(frame_samples as u64)
                    .saturating_add(self.speech_pad_samples);
                return Some(SpeechEvent::Ended { sample_index: end });
            }
        }

        None
    }
}

pub fn expected_silero_frame_size(sample_rate: u32) -> Result<usize, ModelError> {
    match sample_rate {
        8_000 => Ok(SILERO_8KHZ_FRAME_SAMPLES),
        16_000 => Ok(SILERO_16KHZ_FRAME_SAMPLES),
        other => Err(ModelError::UnsupportedSampleRate(other)),
    }
}

pub fn validate_silero_frame(samples: &[f32], sample_rate: u32) -> Result<(), ModelError> {
    let expected = expected_silero_frame_size(sample_rate)?;
    if samples.len() != expected {
        return Err(ModelError::InvalidFrameSize {
            sample_rate,
            expected,
            actual: samples.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_vad_produces_probability() {
        let mut vad = EnergyVad::default();
        let silence = vec![0.0; SILERO_16KHZ_FRAME_SAMPLES];
        let voice = vec![0.25; SILERO_16KHZ_FRAME_SAMPLES];

        assert_eq!(
            vad.speech_probability(&silence, 16_000).expect("silence"),
            0.0
        );
        assert!(vad.speech_probability(&voice, 16_000).expect("voice") > 0.0);
    }

    #[test]
    fn validates_silero_frame_size() {
        let samples = vec![0.0; SILERO_16KHZ_FRAME_SAMPLES - 1];

        let error = validate_silero_frame(&samples, 16_000).expect_err("short frame");

        assert_eq!(
            error,
            ModelError::InvalidFrameSize {
                sample_rate: 16_000,
                expected: SILERO_16KHZ_FRAME_SAMPLES,
                actual: SILERO_16KHZ_FRAME_SAMPLES - 1
            }
        );
    }

    #[test]
    fn segmenter_emits_start_and_end_with_hysteresis() {
        let mut segmenter = SpeechSegmenter::new(16_000, 0.5, 64, 0);

        assert_eq!(
            segmenter.observe(0.7, SILERO_16KHZ_FRAME_SAMPLES),
            Some(SpeechEvent::Started { sample_index: 0 })
        );
        assert_eq!(segmenter.observe(0.4, SILERO_16KHZ_FRAME_SAMPLES), None);
        assert_eq!(segmenter.observe(0.2, SILERO_16KHZ_FRAME_SAMPLES), None);
        assert_eq!(segmenter.observe(0.2, SILERO_16KHZ_FRAME_SAMPLES), None);
        assert_eq!(
            segmenter.observe(0.2, SILERO_16KHZ_FRAME_SAMPLES),
            Some(SpeechEvent::Ended { sample_index: 1024 })
        );
    }
}
