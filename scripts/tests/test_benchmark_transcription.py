import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "benchmark_transcription.py"
SPEC = importlib.util.spec_from_file_location("benchmark_transcription", SCRIPT_PATH)
assert SPEC and SPEC.loader
benchmark = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = benchmark
SPEC.loader.exec_module(benchmark)


class BenchmarkTranscriptionTests(unittest.TestCase):
    def test_report_aggregates_wer_cer_terms_and_latency_without_text(self):
        references = {
            "sample-1": {
                "id": "sample-1",
                "reference_text": "Pháp học cần được kiểm chứng",
                "tags": ["vi", "dharma_talk"],
                "terms": ["Pháp học"],
                "duration_ms": 1000,
            },
            "sample-2": {
                "id": "sample-2",
                "reference_text": "Question and answer",
                "tags": ["online_class", "qa"],
                "duration_ms": 2000,
            },
        }
        hypotheses = {
            "sample-1": {
                "id": "sample-1",
                "hypothesis_text": "Pháp học cần kiểm chứng",
                "latency_ms": 500,
            },
            "sample-2": {
                "id": "sample-2",
                "hypothesis_text": "Question and answer",
                "latency_ms": 1000,
            },
        }

        report = benchmark.build_report(
            references,
            hypotheses,
            run_metadata={"model_id": "fixture-model"},
        )

        self.assertEqual(report["coverage"]["compared_samples"], 2)
        self.assertEqual(report["run"]["model_id"], "fixture-model")
        self.assertEqual(report["overall"]["terms_expected"], 1)
        self.assertEqual(report["overall"]["terms_matched"], 1)
        self.assertIsNotNone(report["overall"]["wer"])
        self.assertEqual(report["by_tag"]["dharma_talk"]["samples"], 1)
        self.assertEqual(report["by_tag"]["online_class"]["samples"], 1)
        self.assertNotIn("reference_text", json.dumps(report, ensure_ascii=False))
        self.assertNotIn("hypothesis_text", json.dumps(report, ensure_ascii=False))

    def test_mismatched_ids_fail_without_explicit_partial_mode(self):
        references = {"sample-1": {"id": "sample-1", "reference_text": "one"}}
        hypotheses = {"sample-2": {"id": "sample-2", "hypothesis_text": "one"}}

        with self.assertRaises(benchmark.BenchmarkInputError):
            benchmark.build_report(references, hypotheses)

    def test_standard_normalization_handles_punctuation_but_keeps_diacritics(self):
        word_metrics, _ = benchmark.calculate_text_metrics(
            "Pháp-học cần kiểm chứng.",
            "pháp học cần kiểm chứng",
            "standard",
        )
        self.assertEqual(word_metrics.error_rate, 0.0)

        diacritic_metrics, _ = benchmark.calculate_text_metrics(
            "Pháp học",
            "Phap hoc",
            "standard",
        )
        self.assertGreater(diacritic_metrics.error_rate or 0.0, 0.0)

    def test_jsonl_reader_rejects_duplicate_ids(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory) / "references.jsonl"
            fixture.write_text(
                '{"id":"sample-1","reference_text":"one"}\n'
                '{"id":"sample-1","reference_text":"two"}\n',
                encoding="utf-8",
            )
            with self.assertRaises(benchmark.BenchmarkInputError):
                benchmark.read_jsonl(fixture, "reference_text")


if __name__ == "__main__":
    unittest.main()
