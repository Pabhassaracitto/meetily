//! Voice activity provider boundary for Meetily's audio pipeline.
//!
//! Silero remains the production implementation. Sherpa is intentionally a
//! requested-but-not-bundled experimental engine until a reviewed model,
//! native artifact, license manifest, and A/B benchmark are shipped together.
//! A Sherpa request therefore falls back to Silero rather than dropping audio.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::quality_profile::QualityProfile;
use super::vad::{ContinuousVadProcessor, SpeechSegment};

const VAD_ENGINE_ENV: &str = "MEETILY_VAD_ENGINE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VadEngine {
    Silero,
    Sherpa,
}

impl VadEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Silero => "silero",
            Self::Sherpa => "sherpa",
        }
    }
}

impl fmt::Display for VadEngine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for VadEngine {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "silero" => Ok(Self::Silero),
            "sherpa" => Ok(Self::Sherpa),
            _ => Err(format!("Unsupported VAD engine '{value}'. Expected silero or sherpa")),
        }
    }
}

/// Serializable status for diagnostics, support, and future settings UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadEngineStatus {
    pub requested_engine: VadEngine,
    pub effective_engine: VadEngine,
    pub fallback_reason: Option<String>,
}

impl VadEngineStatus {
    pub fn did_fallback(&self) -> bool {
        self.requested_engine != self.effective_engine
    }
}

/// Minimal common API used by the live pipeline. A future Sherpa adapter must
/// preserve timestamp semantics: returned segments are 16 kHz mono with
/// recording-relative start/end offsets.
pub trait VoiceActivityProvider: Send {
    fn process_audio(&mut self, samples: &[f32]) -> Result<Vec<SpeechSegment>>;
    fn flush(&mut self) -> Result<Vec<SpeechSegment>>;
    fn status(&self) -> VadEngineStatus;
}

struct SileroVadProvider {
    processor: ContinuousVadProcessor,
    status: VadEngineStatus,
}

impl SileroVadProvider {
    fn new(
        input_sample_rate: u32,
        redemption_time_ms: u32,
        requested_engine: VadEngine,
        fallback_reason: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            processor: ContinuousVadProcessor::new(input_sample_rate, redemption_time_ms)?,
            status: VadEngineStatus {
                requested_engine,
                effective_engine: VadEngine::Silero,
                fallback_reason,
            },
        })
    }
}

impl VoiceActivityProvider for SileroVadProvider {
    fn process_audio(&mut self, samples: &[f32]) -> Result<Vec<SpeechSegment>> {
        self.processor.process_audio(samples)
    }

    fn flush(&mut self) -> Result<Vec<SpeechSegment>> {
        self.processor.flush()
    }

    fn status(&self) -> VadEngineStatus {
        self.status.clone()
    }
}

/// Concrete pipeline provider. Today it owns the proven Silero implementation.
/// Once the Sherpa bridge is genuinely bundled, this enum-like wrapper can add
/// a Sherpa implementation without changing recording/pipeline call sites.
pub struct VadProvider {
    silero: SileroVadProvider,
}

impl VadProvider {
    /// Creates the live provider using the current profile's 400 ms bridge.
    /// Set `MEETILY_VAD_ENGINE=sherpa` only for an engineering smoke test: the
    /// current build reports the fallback explicitly and continues on Silero.
    pub fn for_live_capture(input_sample_rate: u32) -> Result<Self> {
        let config = QualityProfile::LiveBalanced.config();
        let (requested_engine, parse_reason) = requested_engine_from_environment();
        let status = Self::status_for(requested_engine, parse_reason);

        Ok(Self {
            silero: SileroVadProvider::new(
                input_sample_rate,
                config.vad_redemption_ms,
                status.requested_engine,
                status.fallback_reason,
            )?,
        })
    }

    pub fn preview_status() -> VadEngineStatus {
        let (requested_engine, parse_reason) = requested_engine_from_environment();
        Self::status_for(requested_engine, parse_reason)
    }

    fn status_for(
        requested_engine: VadEngine,
        parse_reason: Option<String>,
    ) -> VadEngineStatus {
        match requested_engine {
            VadEngine::Silero => VadEngineStatus {
                requested_engine,
                effective_engine: VadEngine::Silero,
                fallback_reason: parse_reason,
            },
            VadEngine::Sherpa => VadEngineStatus {
                requested_engine,
                effective_engine: VadEngine::Silero,
                fallback_reason: Some(
                    "Sherpa VAD is not bundled in this build; safely using the Silero provider. \
                     Install/approve a pinned Sherpa bridge and model before enabling it."
                        .to_string(),
                ),
            },
        }
    }
}

impl VoiceActivityProvider for VadProvider {
    fn process_audio(&mut self, samples: &[f32]) -> Result<Vec<SpeechSegment>> {
        self.silero.process_audio(samples)
    }

    fn flush(&mut self) -> Result<Vec<SpeechSegment>> {
        self.silero.flush()
    }

    fn status(&self) -> VadEngineStatus {
        self.silero.status()
    }
}

#[tauri::command]
pub fn get_vad_engine_status() -> VadEngineStatus {
    VadProvider::preview_status()
}

fn requested_engine_from_environment() -> (VadEngine, Option<String>) {
    let value = std::env::var(VAD_ENGINE_ENV).ok();
    requested_engine_from_value(value.as_deref())
}

fn requested_engine_from_value(value: Option<&str>) -> (VadEngine, Option<String>) {
    let Some(value) = value else {
        return (VadEngine::Silero, None);
    };

    match value.parse::<VadEngine>() {
        Ok(engine) => (engine, None),
        Err(error) => (
            VadEngine::Silero,
            Some(format!("{error}; safely using the Silero provider.")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{requested_engine_from_value, VadEngine, VadProvider};

    #[test]
    fn parses_supported_vad_engines() {
        assert_eq!("silero".parse::<VadEngine>(), Ok(VadEngine::Silero));
        assert_eq!("SHERPA".parse::<VadEngine>(), Ok(VadEngine::Sherpa));
        assert!("unknown".parse::<VadEngine>().is_err());
    }

    #[test]
    fn sherpa_request_is_explicitly_a_safe_fallback() {
        let (requested_engine, _) = requested_engine_from_value(Some("sherpa"));
        let status = match requested_engine {
            VadEngine::Sherpa => VadProvider::status_for(VadEngine::Sherpa, None),
            VadEngine::Silero => unreachable!("sherpa parser must return sherpa"),
        };

        assert_eq!(status.requested_engine, VadEngine::Sherpa);
        assert_eq!(status.effective_engine, VadEngine::Silero);
        assert!(status.did_fallback());
        assert!(status.fallback_reason.is_some());
    }

    #[test]
    fn invalid_environment_value_falls_back_to_silero() {
        let (engine, reason) = requested_engine_from_value(Some("invalid"));
        assert_eq!(engine, VadEngine::Silero);
        assert!(reason.is_some());
    }
}
