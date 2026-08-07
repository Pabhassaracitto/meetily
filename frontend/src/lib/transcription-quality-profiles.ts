export const QUALITY_PROFILE_IDS = [
  'live_balanced',
  'balanced_batch',
  'high_accuracy_postprocess',
  'long_form_study',
] as const;

export type TranscriptionQualityProfile = (typeof QUALITY_PROFILE_IDS)[number];

export interface QualityProfileOption {
  id: TranscriptionQualityProfile;
  label: string;
  description: string;
  mode: 'live' | 'batch';
  vadRedemptionMs: number;
  maxSegmentSeconds: number;
}

export const QUALITY_PROFILE_OPTIONS: QualityProfileOption[] = [
  {
    id: 'live_balanced',
    label: 'Live balanced',
    description: 'Responsive live capture profile. Keeps the current 400 ms VAD pause bridge.',
    mode: 'live',
    vadRedemptionMs: 400,
    maxSegmentSeconds: 20,
  },
  {
    id: 'balanced_batch',
    label: 'Balanced batch',
    description: 'Faster post-processing with shorter pause bridging for concise recordings.',
    mode: 'batch',
    vadRedemptionMs: 1200,
    maxSegmentSeconds: 20,
  },
  {
    id: 'high_accuracy_postprocess',
    label: 'High-accuracy post-process',
    description: 'Preserves natural pauses and more context. Choose a suitable model separately.',
    mode: 'batch',
    vadRedemptionMs: 2000,
    maxSegmentSeconds: 25,
  },
  {
    id: 'long_form_study',
    label: 'Long-form class / Dharma talk',
    description: 'Most pause-tolerant batch profile for lectures, Q&A, and reflective pauses.',
    mode: 'batch',
    vadRedemptionMs: 2500,
    maxSegmentSeconds: 25,
  },
];

export const DEFAULT_BATCH_QUALITY_PROFILE: TranscriptionQualityProfile = 'high_accuracy_postprocess';
export const DEFAULT_LIVE_QUALITY_PROFILE: TranscriptionQualityProfile = 'live_balanced';

export function isTranscriptionQualityProfile(value: unknown): value is TranscriptionQualityProfile {
  return typeof value === 'string' && (QUALITY_PROFILE_IDS as readonly string[]).includes(value);
}

export function normalizeBatchQualityProfile(value: unknown): TranscriptionQualityProfile {
  if (!isTranscriptionQualityProfile(value)) return DEFAULT_BATCH_QUALITY_PROFILE;
  const option = QUALITY_PROFILE_OPTIONS.find((candidate) => candidate.id === value);
  return option?.mode === 'batch' ? value : DEFAULT_BATCH_QUALITY_PROFILE;
}

export function getQualityProfileOption(value: unknown): QualityProfileOption {
  const normalized = isTranscriptionQualityProfile(value)
    ? value
    : DEFAULT_BATCH_QUALITY_PROFILE;
  return QUALITY_PROFILE_OPTIONS.find((option) => option.id === normalized)
    ?? QUALITY_PROFILE_OPTIONS[0];
}
