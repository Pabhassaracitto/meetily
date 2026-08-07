# Sherpa VAD spike — safe integration boundary

> **Status:** engineering spike foundation; Sherpa model/native artifact is **not bundled or enabled** yet.

## What is implemented now

The live audio pipeline no longer depends directly on `ContinuousVadProcessor`. It goes through `VoiceActivityProvider` / `VadProvider` in `frontend/src-tauri/src/audio/vad_provider.rs`.

- **Production effective engine:** Silero, identical 400 ms live pause bridge.
- **Developer request:** `MEETILY_VAD_ENGINE=sherpa`.
- **Current outcome for that request:** explicit status says `requested=sherpa`, `effective=silero`, with a fallback reason. Recording continues; no audio is discarded.
- **Diagnostic command:** `get_vad_engine_status` is registered in Tauri.

This is intentionally not a marketing claim that Sherpa VAD is active. It creates a stable seam so a real Sherpa provider can replace the fallback without another recording-pipeline rewrite.

## Why Sherpa is not enabled yet

A usable deployment needs more than a Rust crate import:

1. A pinned Sherpa runtime/native library for macOS Apple Silicon, Windows x64 and supported Linux targets.
2. A reviewed VAD model manifest: source, SHA-256, model license, size, sample rate, provider, retention and update policy.
3. A build strategy that does not silently download native libraries/models at customer runtime.
4. A benchmark result against the existing Silero profile for Vietnamese, online class, Dharma talk, low-volume, overlap and long-silence slices.
5. Crash/backpressure/reconnect tests proving fallback does not lose audio.

The current public Rust API for Sherpa VAD exists, but its native build artifact and model are deliberate release inputs, not implicit package-manager downloads. See the [Sherpa Rust VAD API](https://docs.rs/sherpa-onnx/1.13.4/sherpa_onnx/struct.VoiceActivityDetector.html) and [VadModelConfig](https://docs.rs/sherpa-onnx/1.13.4/sherpa_onnx/struct.VadModelConfig.html).

## Required next implementation PR

A real `SherpaVadProvider` must:

```text
1. Load an approved local model by absolute app-data resource path.
2. Receive 16 kHz mono float audio through VoiceActivityProvider.
3. Return the same SpeechSegment timestamp contract as Silero.
4. Emit only aggregate health metrics; never logs audio/transcript content.
5. Fall back to Silero on model load, inference, process, or bridge failures.
6. Write requested/effective engine and model version to processing_runs.
7. Be feature-flagged per session while A/B benchmarking is active.
```

## Rollout gate

Sherpa can be made selectable in customer UI only when all are true:

- benchmark manifest/corpus rights are approved;
- WER/CER and missed-speech/word-boundary metrics are non-regressive versus Silero for the target profile;
- p95 live latency and CPU/RAM budgets are approved for supported hardware;
- model/runtime licenses and NOTICE/SBOM entries are complete;
- failure injection confirms safe fallback;
- privacy/security review confirms no hidden network model/download path.

## Non-goals in this spike

- No automatic recording stop based on VAD.
- No TTS, diarization or voice-cloning dependency on this VAD change.
- No hidden third-party model download or cloud routing.
