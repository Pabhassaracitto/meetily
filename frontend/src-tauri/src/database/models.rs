use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::str::FromStr;

/// The user-facing purpose of a captured session.
///
/// The database stores this as a constrained TEXT value so existing Meetily
/// installations can migrate without changing their meeting IDs or artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    #[default]
    Meeting,
    OnlineClass,
    DharmaTalk,
}

impl SessionType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Meeting => "meeting",
            Self::OnlineClass => "online_class",
            Self::DharmaTalk => "dharma_talk",
        }
    }

    /// Built-in template applied when a new session is created in this mode.
    /// Users can later choose and persist another valid template per session.
    pub const fn default_template_id(self) -> &'static str {
        match self {
            Self::Meeting => "standard_meeting",
            Self::OnlineClass => "online_class",
            Self::DharmaTalk => "dharma_talk",
        }
    }

    /// Use the legacy meeting mode when a caller does not provide a type.
    /// Unknown values are rejected rather than silently being stored as a new
    /// unsupported session category.
    pub fn from_optional(value: Option<&str>) -> Result<Self, String> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => value.parse(),
            None => Ok(Self::default()),
        }
    }
}

impl fmt::Display for SessionType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for SessionType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "meeting" => Ok(Self::Meeting),
            "online_class" => Ok(Self::OnlineClass),
            "dharma_talk" => Ok(Self::DharmaTalk),
            _ => Err(format!(
                "Unsupported session type '{value}'. Expected one of: meeting, online_class, dharma_talk"
            )),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MeetingModel {
    pub id: String,
    pub title: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub folder_path: Option<String>,
    pub session_type: String,
    pub summary_template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct DateTimeUtc(pub DateTime<Utc>);

impl From<NaiveDateTime> for DateTimeUtc {
    fn from(naive: NaiveDateTime) -> Self {
        DateTimeUtc(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
    }
}

// Renamed from TranscriptSegment to Transcript to match the table name
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transcript {
    pub id: String,
    pub meeting_id: String,
    pub transcript: String,
    pub timestamp: String,
    pub summary: Option<String>,
    pub action_items: Option<String>,
    pub key_points: Option<String>,
    // Recording-relative timestamps for audio-transcript synchronization
    pub audio_start_time: Option<f64>,
    pub audio_end_time: Option<f64>,
    pub duration: Option<f64>,
}

/// Non-content metadata captured for a single ASR operation.
///
/// This is supplied by the caller that knows the configuration at the moment
/// processing starts. It deliberately has no fields for transcript/audio text,
/// credentials, prompts, or speaker embeddings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionRunMetadata {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub quality_profile: Option<String>,
    #[serde(default)]
    pub language_hint: Option<String>,
    #[serde(default)]
    pub vad_engine: Option<String>,
    #[serde(default)]
    pub vad_config: Option<serde_json::Value>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub processing_time_ms: Option<i64>,
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,
}

/// Persisted, immutable provenance for a completed transcription operation.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessingRun {
    pub id: String,
    pub meeting_id: String,
    pub run_kind: String,
    pub source_kind: String,
    pub status: String,
    pub provider: String,
    pub model_id: String,
    pub quality_profile: Option<String>,
    pub language_hint: Option<String>,
    pub vad_engine: Option<String>,
    pub vad_config_json: Option<String>,
    pub started_at: String,
    pub completed_at: String,
    pub processing_time_ms: Option<i64>,
    pub metrics_json: Option<String>,
    pub parent_run_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SummaryProcess {
    pub meeting_id: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
    pub result: Option<String>, // JSON
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub chunk_count: i64,
    pub processing_time: f64,
    pub metadata: Option<String>, // JSON
    pub result_backup: Option<String>, // Backup of result before regeneration
    pub result_backup_timestamp: Option<chrono::DateTime<chrono::Utc>>, // When backup was created
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TranscriptChunk {
    pub meeting_id: String,
    pub meeting_name: Option<String>,
    pub transcript_text: String,
    pub model: String,
    pub model_name: String,
    pub chunk_size: Option<i64>,
    pub overlap: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Setting {
    pub id: String,
    pub provider: String,
    pub model: String,
    #[sqlx(rename = "whisperModel")]
    #[serde(rename = "whisperModel")]
    pub whisper_model: String,
    #[sqlx(rename = "groqApiKey")]
    #[serde(rename = "groqApiKey")]
    pub groq_api_key: Option<String>,
    #[sqlx(rename = "openaiApiKey")]
    #[serde(rename = "openaiApiKey")]
    pub openai_api_key: Option<String>,
    #[sqlx(rename = "anthropicApiKey")]
    #[serde(rename = "anthropicApiKey")]
    pub anthropic_api_key: Option<String>,
    #[sqlx(rename = "ollamaApiKey")]
    #[serde(rename = "ollamaApiKey")]
    pub ollama_api_key: Option<String>,
    #[sqlx(rename = "openRouterApiKey")]
    #[serde(rename = "openRouterApiKey")]
    pub open_router_api_key: Option<String>,
    #[sqlx(rename = "ollamaEndpoint")]
    #[serde(rename = "ollamaEndpoint")]
    pub ollama_endpoint: Option<String>,
    /// Custom OpenAI-compatible endpoint configuration stored as JSON
    #[sqlx(rename = "customOpenAIConfig")]
    #[serde(rename = "customOpenAIConfig")]
    pub custom_openai_config: Option<String>,
}

impl Setting {
    /// Parse the custom OpenAI config from JSON string
    pub fn get_custom_openai_config(&self) -> Option<crate::summary::CustomOpenAIConfig> {
        self.custom_openai_config.as_ref().and_then(|json| {
            serde_json::from_str(json).ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SessionType;

    #[test]
    fn session_type_defaults_to_meeting_when_omitted() {
        assert_eq!(
            SessionType::from_optional(None).expect("default session type"),
            SessionType::Meeting
        );
    }

    #[test]
    fn session_type_parses_supported_values() {
        assert_eq!(
            SessionType::from_optional(Some("online_class")).expect("online class"),
            SessionType::OnlineClass
        );
        assert_eq!(
            SessionType::from_optional(Some("dharma_talk")).expect("Dharma talk"),
            SessionType::DharmaTalk
        );
    }

    #[test]
    fn session_type_rejects_unknown_values() {
        assert!(SessionType::from_optional(Some("webinar")).is_err());
    }

    #[test]
    fn session_type_uses_mode_aware_default_templates() {
        assert_eq!(SessionType::Meeting.default_template_id(), "standard_meeting");
        assert_eq!(SessionType::OnlineClass.default_template_id(), "online_class");
        assert_eq!(SessionType::DharmaTalk.default_template_id(), "dharma_talk");
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TranscriptSetting {
    pub id: String,
    pub provider: String,
    pub model: String,
    #[sqlx(rename = "whisperApiKey")]
    #[serde(rename = "whisperApiKey")]
    pub whisper_api_key: Option<String>,
    #[sqlx(rename = "deepgramApiKey")]
    #[serde(rename = "deepgramApiKey")]
    pub deepgram_api_key: Option<String>,
    #[sqlx(rename = "elevenLabsApiKey")]
    #[serde(rename = "elevenLabsApiKey")]
    pub eleven_labs_api_key: Option<String>,
    #[sqlx(rename = "groqApiKey")]
    #[serde(rename = "groqApiKey")]
    pub groq_api_key: Option<String>,
    #[sqlx(rename = "openaiApiKey")]
    #[serde(rename = "openaiApiKey")]
    pub openai_api_key: Option<String>,
}
