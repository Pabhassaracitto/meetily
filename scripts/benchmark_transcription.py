#!/usr/bin/env python3
"""Create a reproducible, privacy-safe ASR benchmark report.

This tool compares a rights-cleared reference JSONL file with a hypothesis JSONL
file. It intentionally does not read audio and never emits transcript text in
its report. That keeps the benchmark artifact safe to attach to a PR while the
underlying corpus remains in approved access-controlled storage.

Reference JSONL schema (one record per line):
  {
    "id": "unique-sample-id",
    "reference_text": "rights-cleared transcript text",
    "tags": ["vi", "online_class"],
    "terms": ["dhamma"],
    "duration_ms": 12000
  }

Hypothesis JSONL schema (one record per line):
  {
    "id": "unique-sample-id",
    "hypothesis_text": "model output",
    "latency_ms": 850
  }

The optional run metadata JSON can describe provider, model hash, VAD settings,
hardware and profile. It is copied into the report without modification.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import statistics
import sys
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence


REPORT_SCHEMA_VERSION = "1.0"


class BenchmarkInputError(ValueError):
    """Raised when benchmark inputs are malformed or incompatible."""


@dataclass(frozen=True)
class EditCounts:
    substitutions: int = 0
    deletions: int = 0
    insertions: int = 0

    @property
    def errors(self) -> int:
        return self.substitutions + self.deletions + self.insertions

    def __add__(self, other: "EditCounts") -> "EditCounts":
        return EditCounts(
            substitutions=self.substitutions + other.substitutions,
            deletions=self.deletions + other.deletions,
            insertions=self.insertions + other.insertions,
        )


@dataclass(frozen=True)
class TextMetrics:
    reference_units: int
    edits: EditCounts

    @property
    def error_rate(self) -> float | None:
        if self.reference_units == 0:
            return None
        return self.edits.errors / self.reference_units


def read_jsonl(path: Path, required_text_key: str) -> dict[str, dict[str, Any]]:
    """Read a JSONL file and validate IDs/text without retaining duplicate rows."""
    rows: dict[str, dict[str, Any]] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise BenchmarkInputError(f"Could not read {path}: {error}") from error

    for line_number, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise BenchmarkInputError(
                f"{path}:{line_number} is not valid JSON: {error.msg}"
            ) from error
        if not isinstance(value, dict):
            raise BenchmarkInputError(f"{path}:{line_number} must be a JSON object")

        sample_id = value.get("id")
        text = value.get(required_text_key)
        if not isinstance(sample_id, str) or not sample_id.strip():
            raise BenchmarkInputError(f"{path}:{line_number} requires a non-empty string id")
        if not isinstance(text, str):
            raise BenchmarkInputError(
                f"{path}:{line_number} requires string field {required_text_key!r}"
            )
        if sample_id in rows:
            raise BenchmarkInputError(f"{path}:{line_number} duplicates id {sample_id!r}")
        rows[sample_id] = value

    if not rows:
        raise BenchmarkInputError(f"{path} contains no benchmark records")
    return rows


def normalize_text(value: str, mode: str) -> str:
    """Normalize text for a comparable but diacritic-preserving error rate.

    ``standard`` uses Unicode NFKC, case folding, punctuation-to-space and
    whitespace collapse. It deliberately retains Vietnamese diacritics and
    letters from Pāli/Sanskrit so spelling quality remains measurable.
    """
    if mode == "strict":
        return value
    if mode != "standard":
        raise BenchmarkInputError(f"Unsupported normalization mode: {mode}")

    normalized = unicodedata.normalize("NFKC", value).casefold()
    normalized = "".join(
        " " if unicodedata.category(character).startswith("P") else character
        for character in normalized
    )
    return " ".join(normalized.split())


def edit_counts(reference: Sequence[str], hypothesis: Sequence[str]) -> EditCounts:
    """Return deterministic substitution/deletion/insertion counts.

    A dynamic-programming cell stores total edit cost plus operation counts.
    The tuple tie-breaker is deterministic, which makes reports stable across
    Python versions while preserving the standard Levenshtein distance.
    """
    previous: list[tuple[int, int, int, int]] = [
        (index, 0, 0, index) for index in range(len(hypothesis) + 1)
    ]

    for ref_index, ref_token in enumerate(reference, start=1):
        current: list[tuple[int, int, int, int]] = [(ref_index, 0, ref_index, 0)]
        for hyp_index, hyp_token in enumerate(hypothesis, start=1):
            if ref_token == hyp_token:
                current.append(previous[hyp_index - 1])
                continue

            substitute = previous[hyp_index - 1]
            delete = previous[hyp_index]
            insert = current[hyp_index - 1]
            candidates = (
                (substitute[0] + 1, substitute[1] + 1, substitute[2], substitute[3]),
                (delete[0] + 1, delete[1], delete[2] + 1, delete[3]),
                (insert[0] + 1, insert[1], insert[2], insert[3] + 1),
            )
            current.append(min(candidates))
        previous = current

    _, substitutions, deletions, insertions = previous[-1]
    return EditCounts(substitutions, deletions, insertions)


def calculate_text_metrics(reference: str, hypothesis: str, mode: str) -> tuple[TextMetrics, TextMetrics]:
    normalized_reference = normalize_text(reference, mode)
    normalized_hypothesis = normalize_text(hypothesis, mode)
    word_reference = normalized_reference.split()
    word_hypothesis = normalized_hypothesis.split()
    char_reference = list(normalized_reference)
    char_hypothesis = list(normalized_hypothesis)
    return (
        TextMetrics(len(word_reference), edit_counts(word_reference, word_hypothesis)),
        TextMetrics(len(char_reference), edit_counts(char_reference, char_hypothesis)),
    )


def percentile(values: Sequence[float], quantile: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * quantile
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return ordered[lower]
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def metric_summary(metrics: Iterable[tuple[TextMetrics, TextMetrics]], term_results: Iterable[tuple[int, int]], latencies: Iterable[float], rtfs: Iterable[float]) -> dict[str, Any]:
    metric_rows = list(metrics)
    term_rows = list(term_results)
    word_reference_units = 0
    word_edits = EditCounts()
    char_reference_units = 0
    char_edits = EditCounts()
    terms_expected = 0
    terms_matched = 0

    for word_metric, char_metric in metric_rows:
        word_reference_units += word_metric.reference_units
        word_edits += word_metric.edits
        char_reference_units += char_metric.reference_units
        char_edits += char_metric.edits

    for expected, matched in term_rows:
        terms_expected += expected
        terms_matched += matched

    latency_values = list(latencies)
    rtf_values = list(rtfs)

    def rate(edits: EditCounts, units: int) -> float | None:
        return edits.errors / units if units else None

    return {
        "samples": len(metric_rows),
        "wer": rate(word_edits, word_reference_units),
        "cer": rate(char_edits, char_reference_units),
        "word_reference_units": word_reference_units,
        "word_edits": {
            "substitutions": word_edits.substitutions,
            "deletions": word_edits.deletions,
            "insertions": word_edits.insertions,
        },
        "character_reference_units": char_reference_units,
        "character_edits": {
            "substitutions": char_edits.substitutions,
            "deletions": char_edits.deletions,
            "insertions": char_edits.insertions,
        },
        "term_accuracy": (terms_matched / terms_expected) if terms_expected else None,
        "terms_expected": terms_expected,
        "terms_matched": terms_matched,
        "latency_ms": {
            "samples": len(latency_values),
            "mean": statistics.fmean(latency_values) if latency_values else None,
            "p50": percentile(latency_values, 0.50),
            "p95": percentile(latency_values, 0.95),
        },
        "real_time_factor": {
            "samples": len(rtf_values),
            "mean": statistics.fmean(rtf_values) if rtf_values else None,
            "p50": percentile(rtf_values, 0.50),
            "p95": percentile(rtf_values, 0.95),
        },
    }


def term_match_counts(reference_record: Mapping[str, Any], normalized_hypothesis: str, mode: str) -> tuple[int, int]:
    terms = reference_record.get("terms", [])
    if terms is None:
        terms = []
    if not isinstance(terms, list) or not all(isinstance(term, str) for term in terms):
        raise BenchmarkInputError(
            f"Reference {reference_record.get('id')!r} has a non-string terms list"
        )

    matched = 0
    for term in terms:
        normalized_term = normalize_text(term, mode)
        if normalized_term and normalized_term in normalized_hypothesis:
            matched += 1
    return len(terms), matched


def build_report(
    references: Mapping[str, Mapping[str, Any]],
    hypotheses: Mapping[str, Mapping[str, Any]],
    *,
    normalization: str = "standard",
    run_metadata: Mapping[str, Any] | None = None,
    allow_partial: bool = False,
) -> dict[str, Any]:
    """Create an aggregate report without exposing reference/hypothesis text."""
    reference_ids = set(references)
    hypothesis_ids = set(hypotheses)
    missing_hypotheses = sorted(reference_ids - hypothesis_ids)
    unexpected_hypotheses = sorted(hypothesis_ids - reference_ids)

    if (missing_hypotheses or unexpected_hypotheses) and not allow_partial:
        raise BenchmarkInputError(
            "Reference/hypothesis IDs differ. "
            f"Missing hypotheses: {missing_hypotheses[:5]}; "
            f"unexpected hypotheses: {unexpected_hypotheses[:5]}. "
            "Use --allow-partial only for an explicitly incomplete run."
        )

    all_metrics: list[tuple[TextMetrics, TextMetrics]] = []
    all_terms: list[tuple[int, int]] = []
    all_latencies: list[float] = []
    all_rtfs: list[float] = []
    by_tag: dict[str, dict[str, list[Any]]] = {}

    for sample_id in sorted(reference_ids & hypothesis_ids):
        reference_record = references[sample_id]
        hypothesis_record = hypotheses[sample_id]
        reference_text = reference_record["reference_text"]
        hypothesis_text = hypothesis_record["hypothesis_text"]
        if not isinstance(reference_text, str) or not isinstance(hypothesis_text, str):
            raise BenchmarkInputError(f"Sample {sample_id!r} has non-string transcript text")

        word_metric, char_metric = calculate_text_metrics(reference_text, hypothesis_text, normalization)
        terms = term_match_counts(
            reference_record,
            normalize_text(hypothesis_text, normalization),
            normalization,
        )
        latency = hypothesis_record.get("latency_ms")
        duration = reference_record.get("duration_ms")
        latency_value = float(latency) if isinstance(latency, (int, float)) and latency >= 0 else None
        duration_value = float(duration) if isinstance(duration, (int, float)) and duration > 0 else None
        rtf = latency_value / duration_value if latency_value is not None and duration_value else None

        all_metrics.append((word_metric, char_metric))
        all_terms.append(terms)
        if latency_value is not None:
            all_latencies.append(latency_value)
        if rtf is not None:
            all_rtfs.append(rtf)

        tags = reference_record.get("tags", [])
        if tags is None:
            tags = []
        if not isinstance(tags, list) or not all(isinstance(tag, str) and tag for tag in tags):
            raise BenchmarkInputError(f"Reference {sample_id!r} has invalid tags")
        for tag in {"all", *tags}:
            bucket = by_tag.setdefault(tag, {"metrics": [], "terms": [], "latencies": [], "rtfs": []})
            bucket["metrics"].append((word_metric, char_metric))
            bucket["terms"].append(terms)
            if latency_value is not None:
                bucket["latencies"].append(latency_value)
            if rtf is not None:
                bucket["rtfs"].append(rtf)

    overall = metric_summary(all_metrics, all_terms, all_latencies, all_rtfs)
    by_tag_summary = {
        tag: metric_summary(
            bucket["metrics"], bucket["terms"], bucket["latencies"], bucket["rtfs"]
        )
        for tag, bucket in sorted(by_tag.items())
    }

    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "generated_at": dt.datetime.now(tz=dt.timezone.utc).isoformat(),
        "normalization": normalization,
        "run": dict(run_metadata or {}),
        "coverage": {
            "reference_samples": len(reference_ids),
            "hypothesis_samples": len(hypothesis_ids),
            "compared_samples": len(all_metrics),
            "missing_hypothesis_ids": missing_hypotheses,
            "unexpected_hypothesis_ids": unexpected_hypotheses,
        },
        "overall": overall,
        "by_tag": by_tag_summary,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--references", type=Path, required=True, help="Rights-cleared reference JSONL path")
    parser.add_argument("--hypotheses", type=Path, required=True, help="ASR hypothesis JSONL path")
    parser.add_argument("--output", type=Path, required=True, help="Aggregate report JSON path")
    parser.add_argument(
        "--run-metadata",
        type=Path,
        help="Optional non-content JSON describing provider/model/VAD/hardware",
    )
    parser.add_argument(
        "--normalization",
        choices=("standard", "strict"),
        default="standard",
        help="Text normalisation policy used before WER/CER (default: standard)",
    )
    parser.add_argument(
        "--allow-partial",
        action="store_true",
        help="Report ID coverage instead of failing when an ASR run is incomplete",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        references = read_jsonl(args.references, "reference_text")
        hypotheses = read_jsonl(args.hypotheses, "hypothesis_text")
        run_metadata: Mapping[str, Any] | None = None
        if args.run_metadata:
            value = json.loads(args.run_metadata.read_text(encoding="utf-8"))
            if not isinstance(value, dict):
                raise BenchmarkInputError("Run metadata must be a JSON object")
            run_metadata = value
        report = build_report(
            references,
            hypotheses,
            normalization=args.normalization,
            run_metadata=run_metadata,
            allow_partial=args.allow_partial,
        )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(
            "Benchmark complete: "
            f"samples={report['coverage']['compared_samples']} "
            f"WER={report['overall']['wer']} CER={report['overall']['cer']}"
        )
        return 0
    except (BenchmarkInputError, json.JSONDecodeError, OSError) as error:
        print(f"benchmark failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
