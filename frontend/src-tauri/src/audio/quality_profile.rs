use std::fmt;
use std::str::FromStr;

/// Reproducible, user-visible processing intent for ASR/VAD batch work.
///
/// Profiles deliberately tune segmentation only. They never silently change
/// the model/provider selected by the user; model choice remains a separate,
/// explicit quality and privacy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityProfile {
    /// Current live pipeline behavior: responsive VAD suitable for capture.
    LiveBalanced,
    /// Faster batch processing with shorter silence bridging.
    BalancedBatch,
    /// Existing Meetily batch behavior: preserve natural pauses and context.
    HighAccuracyPostprocess,
    /// Long-form lecture/Dharma-talk processing with more pause tolerance.
    LongFormStudy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityProfileConfig {
    pub vad_redemption_ms: u32,
    pub max_segment_seconds: usize,
    pub mode: &'static str,
}

impl QualityProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiveBalanced => "live_balanced",
            Self::BalancedBatch => "balanced_batch",
            Self::HighAccuracyPostprocess => "high_accuracy_postprocess",
            Self::LongFormStudy => "long_form_study",
        }
    }

    pub const fn config(self) -> QualityProfileConfig {
        match self {
            Self::LiveBalanced => QualityProfileConfig {
                vad_redemption_ms: 400,
                max_segment_seconds: 20,
                mode: "live",
            },
            Self::BalancedBatch => QualityProfileConfig {
                vad_redemption_ms: 1200,
                max_segment_seconds: 20,
                mode: "batch",
            },
            Self::HighAccuracyPostprocess => QualityProfileConfig {
                vad_redemption_ms: 2000,
                max_segment_seconds: 25,
                mode: "batch",
            },
            Self::LongFormStudy => QualityProfileConfig {
                vad_redemption_ms: 2500,
                max_segment_seconds: 25,
                mode: "batch",
            },
        }
    }

    /// Preserve the pre-Pro batch behavior for old import/retranscribe callers
    /// that do not send a profile.
    pub fn from_optional(value: Option<&str>) -> Result<Self, String> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => value.parse(),
            None => Ok(Self::HighAccuracyPostprocess),
        }
    }
}

impl fmt::Display for QualityProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for QualityProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "live_balanced" => Ok(Self::LiveBalanced),
            "balanced_batch" => Ok(Self::BalancedBatch),
            "high_accuracy_postprocess" => Ok(Self::HighAccuracyPostprocess),
            "long_form_study" => Ok(Self::LongFormStudy),
            _ => Err(format!(
                "Unsupported quality profile '{value}'. Expected live_balanced, balanced_batch, high_accuracy_postprocess, or long_form_study"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::QualityProfile;

    #[test]
    fn defaults_to_current_high_accuracy_batch_behavior() {
        assert_eq!(
            QualityProfile::from_optional(None).expect("default profile"),
            QualityProfile::HighAccuracyPostprocess
        );
    }

    #[test]
    fn long_form_profile_preserves_pauses_longer_than_balanced_batch() {
        let balanced = QualityProfile::BalancedBatch.config();
        let long_form = QualityProfile::LongFormStudy.config();
        assert!(long_form.vad_redemption_ms > balanced.vad_redemption_ms);
    }

    #[test]
    fn rejects_unknown_quality_profiles() {
        assert!(QualityProfile::from_optional(Some("maximum")).is_err());
    }
}
