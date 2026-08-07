import sqlite3
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MIGRATIONS_DIR = REPO_ROOT / "frontend" / "src-tauri" / "migrations"


def migrated_connection() -> sqlite3.Connection:
    connection = sqlite3.connect(":memory:")
    connection.execute("PRAGMA foreign_keys = ON")
    for migration in sorted(MIGRATIONS_DIR.glob("*.sql")):
        connection.executescript(migration.read_text(encoding="utf-8"))
    return connection


class SqliteMigrationTests(unittest.TestCase):
    def test_session_and_template_defaults_are_applied_to_legacy_rows(self):
        connection = migrated_connection()
        self.addCleanup(connection.close)

        connection.execute(
            "INSERT INTO meetings (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
            ("legacy", "Legacy", "2026-08-08T00:00:00Z", "2026-08-08T00:00:00Z"),
        )
        row = connection.execute(
            "SELECT session_type, summary_template_id FROM meetings WHERE id = 'legacy'"
        ).fetchone()
        self.assertEqual(row, ("meeting", "standard_meeting"))

    def test_processing_run_is_deleted_with_its_session(self):
        connection = migrated_connection()
        self.addCleanup(connection.close)

        connection.execute(
            """INSERT INTO meetings (
                id, title, created_at, updated_at, session_type, summary_template_id
            ) VALUES (?, ?, ?, ?, ?, ?)""",
            (
                "session-1",
                "Class",
                "2026-08-08T00:00:00Z",
                "2026-08-08T00:00:00Z",
                "online_class",
                "online_class",
            ),
        )
        connection.execute(
            """INSERT INTO processing_runs (
                id, meeting_id, run_kind, source_kind, status, provider, model_id,
                quality_profile, started_at, completed_at, created_at, processing_time_ms, metrics_json
            ) VALUES (?, ?, 'transcription', 'import', 'completed', ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                "run-1",
                "session-1",
                "localWhisper",
                "large-v3",
                "high_accuracy_postprocess",
                "2026-08-08T00:00:00Z",
                "2026-08-08T00:00:05Z",
                "2026-08-08T00:00:05Z",
                5000,
                '{"segments_transcribed": 12}',
            ),
        )

        self.assertEqual(
            connection.execute("SELECT quality_profile FROM processing_runs WHERE id = 'run-1'").fetchone(),
            ("high_accuracy_postprocess",),
        )

        connection.execute("DELETE FROM meetings WHERE id = 'session-1'")
        self.assertEqual(
            connection.execute("SELECT COUNT(*) FROM processing_runs").fetchone()[0],
            0,
        )

    def test_processing_run_constraints_reject_unknown_sources(self):
        connection = migrated_connection()
        self.addCleanup(connection.close)
        connection.execute(
            "INSERT INTO meetings (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
            ("session-2", "Meeting", "2026-08-08T00:00:00Z", "2026-08-08T00:00:00Z"),
        )

        with self.assertRaises(sqlite3.IntegrityError):
            connection.execute(
                """INSERT INTO processing_runs (
                    id, meeting_id, run_kind, source_kind, status, provider, model_id,
                    started_at, completed_at, created_at
                ) VALUES (?, ?, 'transcription', 'unsupported', 'completed', ?, ?, ?, ?, ?)""",
                (
                    "bad-run",
                    "session-2",
                    "unknown",
                    "unknown",
                    "2026-08-08T00:00:00Z",
                    "2026-08-08T00:00:00Z",
                    "2026-08-08T00:00:00Z",
                ),
            )


if __name__ == "__main__":
    unittest.main()
