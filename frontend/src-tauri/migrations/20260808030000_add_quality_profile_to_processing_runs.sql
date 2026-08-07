-- Preserve the quality profile that produced a transcription run. Existing
-- provenance remains valid and reports NULL when it predates profile support.
ALTER TABLE processing_runs ADD COLUMN quality_profile TEXT;

CREATE INDEX IF NOT EXISTS idx_processing_runs_quality_profile
ON processing_runs(quality_profile);
