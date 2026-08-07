"use client";

import { BookOpen, BriefcaseBusiness, HeartHandshake } from 'lucide-react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  SessionType,
  SESSION_TYPE_OPTIONS,
  getSessionTypeOption,
} from '@/lib/session-types';

interface SessionTypeSelectorProps {
  value: SessionType;
  onValueChange: (value: SessionType) => void;
  disabled?: boolean;
  compact?: boolean;
  id?: string;
}

const ICONS = {
  meeting: BriefcaseBusiness,
  online_class: BookOpen,
  dharma_talk: HeartHandshake,
};

export function SessionTypeSelector({
  value,
  onValueChange,
  disabled = false,
  compact = false,
  id,
}: SessionTypeSelectorProps) {
  const selected = getSessionTypeOption(value);
  const SelectedIcon = ICONS[selected.value];

  return (
    <div className={compact ? 'min-w-[150px]' : 'space-y-1'}>
      {!compact && (
        <label htmlFor={id} className="text-sm font-medium text-gray-700">
          Session type
        </label>
      )}
      <Select
        value={value}
        onValueChange={(nextValue) => onValueChange(nextValue as SessionType)}
        disabled={disabled}
      >
        <SelectTrigger id={id} className={compact ? 'h-10 min-w-[150px]' : 'w-full'} aria-label="Session type">
          <SelectValue>
            <span className="flex items-center gap-2">
              <SelectedIcon className="h-4 w-4 text-blue-600" />
              {compact ? selected.shortLabel : selected.label}
            </span>
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {SESSION_TYPE_OPTIONS.map((option) => {
            const Icon = ICONS[option.value];
            return (
              <SelectItem key={option.value} value={option.value}>
                <span className="flex items-center gap-2">
                  <Icon className="h-4 w-4 text-blue-600" />
                  <span>{option.label}</span>
                </span>
              </SelectItem>
            );
          })}
        </SelectContent>
      </Select>
      {!compact && <p className="text-xs text-muted-foreground">{selected.description}</p>}
    </div>
  );
}
