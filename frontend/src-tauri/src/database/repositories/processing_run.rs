use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Error as SqlxError, SqliteConnection, SqlitePool};
use uuid::Uuid;

use crate::database::models::{ProcessingRun, TranscriptionRunMetadata};

const MAX_FIELD_LENGTH: usize = 255;
const MAX_LANGUAGE_LENGTH: usize = 64;
const MAX_JSON_BYTES: usize = 32 * 1024;
const MAX_PROCESSING_TIME_MS: i64 = 30 * 24 * 60 * 60 * 1000;

pub struct ProcessingRunsRepository;

impl ProcessingRunsRepository {
    /// Inserts a completed ASR provenance record using the caller's existing
    /// transaction. Keeping it in the same transaction as transcript writes
    /// prevents a saved session from claiming a processing run that never
    /// completed (or vice versa).
    pub async fn insert_completed_transcription_run(
        connection: &mut SqliteConnection,
        meeting_id: &str,
        source_kind: &str,
        metadata: Option<&TranscriptionRunMetadata>,
        parent_run_id: Option<&str>,
    ) -> Result<String, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(protocol_error("meeting_id cannot be empty"));
        }
        validate_source_kind(source_kind)?;

        let now = Utc::now();
        let provider = required_field(
            metadata.and_then(|value| value.provider.as_deref()),
            "provider",
            "unknown",
            MAX_FIELD_LENGTH,
        )?;
        let model_id = required_field(
            metadata.and_then(|value| value.model_id.as_deref()),
            "model_id",
            "unknown",
            MAX_FIELD_LENGTH,
        )?;
        let quality_profile = optional_field(
            metadata.and_then(|value| value.quality_profile.as_deref()),
            "quality_profile",
            MAX_FIELD_LENGTH,
        )?;
        let language_hint = optional_field(
            metadata.and_then(|value| value.language_hint.as_deref()),
            "language_hint",
            MAX_LANGUAGE_LENGTH,
        )?;
        let vad_engine = optional_field(
            metadata.and_then(|value| value.vad_engine.as_deref()),
            "vad_engine",
            MAX_FIELD_LENGTH,
        )?;
        let vad_config_json = optional_safe_json(
            metadata.and_then(|value| value.vad_config.as_ref()),
            "vad_config",
        )?;
        let metrics_json = optional_safe_json(
            metadata.and_then(|value| value.metrics.as_ref()),
            "metrics",
        )?;
        let started_at = normalized_timestamp(
            metadata.and_then(|value| value.started_at.as_deref()),
            now.clone(),
        )?;
        let processing_time_ms = metadata
            .and_then(|value| value.processing_time_ms)
            .map(validate_processing_time)
            .transpose()?;
        let parent_run_id = optional_field(parent_run_id, "parent_run_id", MAX_FIELD_LENGTH)?;
        let run_id = format!("run-{}", Uuid::new_v4());
        let completed_at = now.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO processing_runs (
                id, meeting_id, run_kind, source_kind, status, provider, model_id,
                quality_profile, language_hint, vad_engine, vad_config_json, started_at,
                completed_at, processing_time_ms, metrics_json, parent_run_id, created_at
            ) VALUES (?, ?, 'transcription', ?, 'completed', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&run_id)
        .bind(meeting_id)
        .bind(source_kind)
        .bind(provider)
        .bind(model_id)
        .bind(quality_profile)
        .bind(language_hint)
        .bind(vad_engine)
        .bind(vad_config_json)
        .bind(started_at)
        .bind(&completed_at)
        .bind(processing_time_ms)
        .bind(metrics_json)
        .bind(parent_run_id)
        .bind(&completed_at)
        .execute(connection)
        .await?;

        Ok(run_id)
    }

    pub async fn list_for_meeting(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Vec<ProcessingRun>, SqlxError> {
        if meeting_id.trim().is_empty() {
            return Err(protocol_error("meeting_id cannot be empty"));
        }

        sqlx::query_as::<_, ProcessingRun>(
            "SELECT * FROM processing_runs WHERE meeting_id = ? ORDER BY created_at DESC",
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await
    }
}

fn validate_source_kind(source_kind: &str) -> Result<(), SqlxError> {
    match source_kind {
        "live" | "import" | "retranscription" | "recovery" => Ok(()),
        _ => Err(protocol_error(format!("Unsupported processing source: {source_kind}"))),
    }
}

fn required_field(
    value: Option<&str>,
    field: &str,
    fallback: &str,
    max_length: usize,
) -> Result<String, SqlxError> {
    match optional_field(value, field, max_length)? {
        Some(value) => Ok(value),
        None => Ok(fallback.to_string()),
    }
}

fn optional_field(
    value: Option<&str>,
    field: &str,
    max_length: usize,
) -> Result<Option<String>, SqlxError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if value.len() > max_length {
        return Err(protocol_error(format!(
            "{field} exceeds the maximum length of {max_length} characters"
        )));
    }
    Ok(Some(value.to_string()))
}

fn normalized_timestamp(value: Option<&str>, fallback: DateTime<Utc>) -> Result<String, SqlxError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(fallback.to_rfc3339());
    };

    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc).to_rfc3339())
        .map_err(|_| protocol_error("started_at must be RFC 3339"))
}

fn validate_processing_time(value: i64) -> Result<i64, SqlxError> {
    if !(0..=MAX_PROCESSING_TIME_MS).contains(&value) {
        return Err(protocol_error(format!(
            "processing_time_ms must be between 0 and {MAX_PROCESSING_TIME_MS}"
        )));
    }
    Ok(value)
}

fn optional_safe_json(value: Option<&Value>, field: &str) -> Result<Option<String>, SqlxError> {
    let Some(value) = value else {
        return Ok(None);
    };

    reject_content_fields(value, field)?;
    let serialized = serde_json::to_string(value)
        .map_err(|error| protocol_error(format!("Failed to serialize {field}: {error}")))?;
    if serialized.len() > MAX_JSON_BYTES {
        return Err(protocol_error(format!(
            "{field} exceeds the maximum size of {MAX_JSON_BYTES} bytes"
        )));
    }
    Ok(Some(serialized))
}

/// Metrics/config are intentionally non-content. Reject common payload keys so
/// a future caller cannot accidentally turn provenance into a transcript store.
fn reject_content_fields(value: &Value, field: &str) -> Result<(), SqlxError> {
    const PROHIBITED_KEYS: [&str; 10] = [
        "audio",
        "api_key",
        "apikey",
        "embedding",
        "prompt",
        "sample_data",
        "speaker_embedding",
        "text",
        "transcript",
        "voice",
    ];

    match value {
        Value::Object(object) => {
            for (key, nested_value) in object {
                let normalized_key = key.to_ascii_lowercase();
                if PROHIBITED_KEYS.iter().any(|prohibited_key| {
                    normalized_key == *prohibited_key
                        || normalized_key.starts_with(&format!("{prohibited_key}_"))
                }) {
                    return Err(protocol_error(format!(
                        "{field} must not contain content field '{key}'"
                    )));
                }
                reject_content_fields(nested_value, field)?;
            }
        }
        Value::Array(values) => {
            for nested_value in values {
                reject_content_fields(nested_value, field)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn protocol_error(message: impl Into<String>) -> SqlxError {
    SqlxError::Protocol(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_allows_aggregate_metrics() {
        let value = serde_json::json!({
            "segments_detected": 12,
            "average_confidence": 0.91,
            "vad": { "redemption_ms": 2000 }
        });
        assert!(optional_safe_json(Some(&value), "metrics").is_ok());
    }

    #[test]
    fn metadata_rejects_transcript_content() {
        let value = serde_json::json!({ "transcript_excerpt": "must not be stored" });
        assert!(optional_safe_json(Some(&value), "metrics").is_err());
    }

    #[test]
    fn timestamp_is_normalized_to_utc() {
        assert_eq!(
            normalized_timestamp(Some("2026-08-08T12:00:00+05:30"), Utc::now())
                .expect("valid timestamp"),
            "2026-08-08T06:30:00+00:00"
        );
    }

    #[test]
    fn only_supported_sources_are_accepted() {
        assert!(validate_source_kind("live").is_ok());
        assert!(validate_source_kind("retranscription").is_ok());
        assert!(validate_source_kind("sherpa").is_err());
    }
}
