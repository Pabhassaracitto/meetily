import type { TranscriptModelProps } from '@/components/TranscriptSettings';

export const ACTIVE_TRANSCRIPTION_RUN_METADATA_KEY = 'active_transcription_run_metadata';

export type ProcessingSourceKind = 'live' | 'import' | 'retranscription' | 'recovery';

/**
 * Non-content metadata persisted beside a completed ASR run. Do not add
 * transcript text, audio samples, credentials, prompts, or voice embeddings.
 */
export interface TranscriptionRunMetadata {
  provider?: string | null;
  modelId?: string | null;
  languageHint?: string | null;
  vadEngine?: string | null;
  vadConfig?: Record<string, unknown> | null;
  startedAt?: string | null;
  processingTimeMs?: number | null;
  metrics?: Record<string, number | string | boolean | null> | null;
}

/** Tauri/Rust response shape for a persisted, non-content processing run. */
export interface ProcessingRun {
  id: string;
  meeting_id: string;
  run_kind: 'transcription';
  source_kind: ProcessingSourceKind;
  status: 'completed';
  provider: string;
  model_id: string;
  language_hint?: string | null;
  vad_engine?: string | null;
  vad_config_json?: string | null;
  started_at: string;
  completed_at: string;
  processing_time_ms?: number | null;
  metrics_json?: string | null;
  parent_run_id?: string | null;
  created_at: string;
}

export function formatProcessingDuration(value: number | null | undefined): string | null {
  if (value === null || value === undefined || value < 0) return null;
  if (value < 1000) return `${Math.round(value)} ms`;
  if (value < 60_000) return `${(value / 1000).toFixed(value < 10_000 ? 1 : 0)} s`;
  return `${Math.floor(value / 60_000)}m ${Math.round((value % 60_000) / 1000)}s`;
}

export function createLiveTranscriptionRunMetadata(
  config: Pick<TranscriptModelProps, 'provider' | 'model'>,
  selectedLanguage: string
): TranscriptionRunMetadata {
  return {
    provider: config.provider || 'unknown',
    modelId: config.model || 'unknown',
    languageHint: selectedLanguage && selectedLanguage !== 'auto' ? selectedLanguage : null,
    vadEngine: 'silero',
    vadConfig: {
      mode: 'live',
      redemptionMs: 400,
      sampleRateHz: 16000,
    },
    startedAt: new Date().toISOString(),
  };
}

export function saveActiveTranscriptionRunMetadata(metadata: TranscriptionRunMetadata): void {
  sessionStorage.setItem(ACTIVE_TRANSCRIPTION_RUN_METADATA_KEY, JSON.stringify(metadata));
}

export function readActiveTranscriptionRunMetadata(): TranscriptionRunMetadata | undefined {
  const raw = sessionStorage.getItem(ACTIVE_TRANSCRIPTION_RUN_METADATA_KEY);
  if (!raw) return undefined;

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return undefined;
    return parsed as TranscriptionRunMetadata;
  } catch {
    return undefined;
  }
}

export function finalizeTranscriptionRunMetadata(
  metadata: TranscriptionRunMetadata | undefined,
  completedAtMs: number = Date.now()
): TranscriptionRunMetadata | undefined {
  if (!metadata) return undefined;

  const startedAtMs = metadata.startedAt ? Date.parse(metadata.startedAt) : Number.NaN;
  const processingTimeMs = Number.isFinite(startedAtMs)
    ? Math.max(0, Math.round(completedAtMs - startedAtMs))
    : undefined;

  return {
    ...metadata,
    processingTimeMs,
  };
}

export function clearActiveTranscriptionRunMetadata(): void {
  sessionStorage.removeItem(ACTIVE_TRANSCRIPTION_RUN_METADATA_KEY);
}
