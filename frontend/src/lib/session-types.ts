export const SESSION_TYPES = ['meeting', 'online_class', 'dharma_talk'] as const;

export type SessionType = (typeof SESSION_TYPES)[number];

export interface SessionTypeOption {
  value: SessionType;
  label: string;
  shortLabel: string;
  description: string;
  defaultTemplateId: string;
}

export const SESSION_TYPE_OPTIONS: SessionTypeOption[] = [
  {
    value: 'meeting',
    label: 'Meeting',
    shortLabel: 'Meeting',
    description: 'Decisions, action items, and discussion notes.',
    defaultTemplateId: 'standard_meeting',
  },
  {
    value: 'online_class',
    label: 'Online Class',
    shortLabel: 'Class',
    description: 'Lesson outline, concepts, Q&A, and study tasks.',
    defaultTemplateId: 'online_class',
  },
  {
    value: 'dharma_talk',
    label: 'Dharma Talk',
    shortLabel: 'Dharma',
    description: 'Evidence-aware notes, terms, excerpts, and reflections.',
    defaultTemplateId: 'dharma_talk',
  },
];

export const DEFAULT_SESSION_TYPE: SessionType = 'meeting';

export function isSessionType(value: unknown): value is SessionType {
  return typeof value === 'string' && (SESSION_TYPES as readonly string[]).includes(value);
}

export function normalizeSessionType(value: unknown): SessionType {
  return isSessionType(value) ? value : DEFAULT_SESSION_TYPE;
}

export function getSessionTypeOption(value: unknown): SessionTypeOption {
  const normalized = normalizeSessionType(value);
  return SESSION_TYPE_OPTIONS.find((option) => option.value === normalized) ?? SESSION_TYPE_OPTIONS[0];
}

export function getDefaultTemplateForSessionType(value: unknown): string {
  return getSessionTypeOption(value).defaultTemplateId;
}
