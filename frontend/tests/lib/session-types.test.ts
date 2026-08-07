import { describe, expect, test } from "bun:test";
import {
  DEFAULT_SESSION_TYPE,
  getDefaultTemplateForSessionType,
  getSessionTypeOption,
  normalizeSessionType,
} from "../../src/lib/session-types";

describe("session types", () => {
  test("defaults unknown values to a conventional meeting", () => {
    expect(normalizeSessionType(undefined)).toBe(DEFAULT_SESSION_TYPE);
    expect(normalizeSessionType("unsupported")).toBe(DEFAULT_SESSION_TYPE);
  });

  test("maps class and Dharma modes to their evidence-aware templates", () => {
    expect(getDefaultTemplateForSessionType("online_class")).toBe("online_class");
    expect(getDefaultTemplateForSessionType("dharma_talk")).toBe("dharma_talk");
  });

  test("returns useful mode metadata for the UI", () => {
    expect(getSessionTypeOption("dharma_talk")).toMatchObject({
      label: "Dharma Talk",
      shortLabel: "Dharma",
    });
  });
});
