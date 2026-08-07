-- Meetily Pro foundation: distinguish a conventional meeting from an online class
-- or Dharma talk while retaining full compatibility with existing meeting rows.
--
-- Existing records receive the default `meeting` type. Keep this migration
-- additive: recordings and transcripts remain attached to the original IDs.
ALTER TABLE meetings
ADD COLUMN session_type TEXT NOT NULL DEFAULT 'meeting'
CHECK (session_type IN ('meeting', 'online_class', 'dharma_talk'));

CREATE INDEX IF NOT EXISTS idx_meetings_session_type_created_at
ON meetings(session_type, created_at DESC);
