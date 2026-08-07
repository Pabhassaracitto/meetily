-- Meetily Pro foundation: immutable provenance for transcription/retranscription.
-- This table stores configuration and aggregate metrics only. Do not store raw
-- audio, transcript text, prompts, API keys, or speaker embeddings here.
CREATE TABLE IF NOT EXISTS processing_runs (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL,
    run_kind TEXT NOT NULL CHECK (run_kind IN ('transcription')),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('live', 'import', 'retranscription', 'recovery')),
    status TEXT NOT NULL CHECK (status IN ('completed')),
    provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    language_hint TEXT,
    vad_engine TEXT,
    vad_config_json TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    processing_time_ms INTEGER,
    metrics_json TEXT,
    parent_run_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_run_id) REFERENCES processing_runs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_processing_runs_meeting_created
ON processing_runs(meeting_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_processing_runs_parent
ON processing_runs(parent_run_id);
