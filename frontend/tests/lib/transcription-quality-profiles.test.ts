import { describe, expect, test } from "bun:test";
import {
  DEFAULT_BATCH_QUALITY_PROFILE,
  getQualityProfileOption,
  normalizeBatchQualityProfile,
} from "../../src/lib/transcription-quality-profiles";

describe("transcription quality profiles", () => {
  test("defaults legacy batch callers to the current high-accuracy behavior", () => {
    expect(normalizeBatchQualityProfile(undefined)).toBe(DEFAULT_BATCH_QUALITY_PROFILE);
    expect(getQualityProfileOption(DEFAULT_BATCH_QUALITY_PROFILE)).toMatchObject({
      vadRedemptionMs: 2000,
      maxSegmentSeconds: 25,
      mode: "batch",
    });
  });

  test("does not allow the live profile in batch import or retranscription", () => {
    expect(normalizeBatchQualityProfile("live_balanced")).toBe(DEFAULT_BATCH_QUALITY_PROFILE);
  });

  test("provides a pause-tolerant profile for long-form study", () => {
    const profile = getQualityProfileOption("long_form_study");
    expect(profile.vadRedemptionMs).toBeGreaterThan(
      getQualityProfileOption("balanced_batch").vadRedemptionMs,
    );
  });
});
