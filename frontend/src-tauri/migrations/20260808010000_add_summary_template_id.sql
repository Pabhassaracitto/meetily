-- Meetily Pro foundation: preserve the summary workflow selected for a session.
-- Existing meetings retain the community default and new sessions receive a
-- type-aware default in the repository layer.
ALTER TABLE meetings
ADD COLUMN summary_template_id TEXT NOT NULL DEFAULT 'standard_meeting';

-- Sessions created after the session-type migration but before this migration
-- did not yet have a persisted workflow. Give them the mode-aware default.
UPDATE meetings
SET summary_template_id = CASE session_type
    WHEN 'online_class' THEN 'online_class'
    WHEN 'dharma_talk' THEN 'dharma_talk'
    ELSE 'standard_meeting'
END;

CREATE INDEX IF NOT EXISTS idx_meetings_summary_template_id
ON meetings(summary_template_id);
