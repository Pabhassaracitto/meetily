import { describe, expect, test } from "bun:test";
import {
  createLiveTranscriptionRunMetadata,
  finalizeTranscriptionRunMetadata,
  formatProcessingDuration,
} from "../../src/lib/processing-runs";

describe("transcription processing provenance", () => {
  test("snapshots the live local ASR and VAD configuration without content", () => {
    const metadata = createLiveTranscriptionRunMetadata(
      { provider: "parakeet", model: "parakeet-tdt-0.6b-v3-int8" },
      "auto",
    );

    expect(metadata).toMatchObject({
      provider: "parakeet",
      modelId: "parakeet-tdt-0.6b-v3-int8",
      languageHint: null,
      vadEngine: "silero",
      vadConfig: {
        mode: "live",
        redemptionMs: 400,
        sampleRateHz: 16000,
      },
    });
    expect(Object.keys(metadata)).not.toContain("transcript");
    expect(Object.keys(metadata)).not.toContain("audio");
  });

  test("calculates elapsed processing time from the start snapshot", () => {
    const metadata = finalizeTranscriptionRunMetadata(
      { startedAt: "2026-08-08T00:00:00.000Z", provider: "localWhisper" },
      Date.parse("2026-08-08T00:00:02.250Z"),
    );

    expect(metadata?.processingTimeMs).toBe(2250);
  });

  test("formats non-content processing duration for provenance UI", () => {
    expect(formatProcessingDuration(250)).toBe("250 ms");
    expect(formatProcessingDuration(2250)).toBe("2.3 s");
    expect(formatProcessingDuration(undefined)).toBeNull();
  });
});
