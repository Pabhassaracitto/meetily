# Meetily transcription benchmark protocol

This directory contains the **contract** for measuring transcription quality. It must not become a repository of customer recordings, unlicensed Dharma talks, classroom recordings, raw transcript text, voice samples, or API credentials.

## Privacy and rights rules

1. Store full audio and reference transcripts only in approved access-controlled storage.
2. Each corpus item needs a rights/consent record, expiry/retention rule, source, language, content mode and reviewer.
3. Git may contain synthetic fixtures and aggregate reports only. `benchmarks/.gitignore` blocks common raw asset/report paths by default.
4. A benchmark report must contain metrics, model/run metadata and opaque sample IDs — **never** reference or hypothesis text.
5. Do not enrol a voice model, identify a speaker or create a clone profile from benchmark material unless that is separately authorized. ASR quality consent is not voice-cloning consent.

## Files and contracts

- [`manifest.schema.json`](./manifest.schema.json): metadata contract for a private corpus manifest.
- [`../scripts/benchmark_transcription.py`](../scripts/benchmark_transcription.py): stdlib-only WER/CER, term accuracy, latency and real-time-factor reporter.
- `references.jsonl` and `hypotheses.jsonl`: kept outside Git in approved storage.

### Reference JSONL

```json
{"id":"opaque-vi-class-001","reference_text":"Nội dung đã được cấp quyền", "tags":["vi","online_class"], "terms":["định nghĩa"], "duration_ms":12000}
```

### Hypothesis JSONL

```json
{"id":"opaque-vi-class-001","hypothesis_text":"Nội dung ASR", "latency_ms":850}
```

`id` values must match. The default run fails when either side has missing/unknown IDs. Use `--allow-partial` only for an explicitly incomplete run; the report will expose coverage gaps.

## Run a comparison

```bash
python3 scripts/benchmark_transcription.py \
  --references /approved-corpus/references.jsonl \
  --hypotheses /approved-runs/whisper-large-v3.jsonl \
  --run-metadata /approved-runs/whisper-large-v3.run.json \
  --output /approved-reports/whisper-large-v3.json
```

Example non-content `run-metadata`:

```json
{
  "run_id": "2026-08-08-whisper-large-v3-vn-long-form",
  "provider": "localWhisper",
  "model_id": "large-v3",
  "model_sha256": "...",
  "vad_engine": "silero",
  "vad_config": {"redemption_ms": 2000},
  "profile": "high_accuracy_postprocess",
  "hardware": "approved test machine"
}
```

## Required slices before a professional-accuracy claim

Report aggregate and per-tag metrics for at least:

| Tag | Why it matters |
|---|---|
| `vi` | Vietnamese baseline; retain diacritics in normalisation. |
| `online_class` | Explanations, terms, questions, demonstrations and homework. |
| `dharma_talk` | Long-form single-speaker talk, Pāli/Sanskrit/Hán–Việt terms and reflective pauses. |
| `qa` | Turn-taking and short questions/responses. |
| `noise` / `low_volume` | Realistic microphone/system-audio quality. |
| `overlap` | Acknowledges the hard diarization/ASR case rather than hiding it. |
| `long_silence` | Confirms VAD does not lose quiet or contemplative content. |

Minimum report fields are WER, CER, term accuracy, sample coverage, latency p50/p95 and real-time factor. Choose release thresholds only after the first baseline is produced; do not invent a percentage before the corpus exists.
