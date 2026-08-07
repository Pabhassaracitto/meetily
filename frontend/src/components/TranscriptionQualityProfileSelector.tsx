"use client";

import { Gauge } from 'lucide-react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  getQualityProfileOption,
  QUALITY_PROFILE_OPTIONS,
  TranscriptionQualityProfile,
} from '@/lib/transcription-quality-profiles';

interface TranscriptionQualityProfileSelectorProps {
  value: TranscriptionQualityProfile;
  onValueChange: (value: TranscriptionQualityProfile) => void;
  disabled?: boolean;
  id?: string;
}

/**
 * Batch-only selector. Model/provider remain independently selected so a
 * profile never silently changes the user's data route or model choice.
 */
export function TranscriptionQualityProfileSelector({
  value,
  onValueChange,
  disabled = false,
  id,
}: TranscriptionQualityProfileSelectorProps) {
  const selected = getQualityProfileOption(value);
  const batchProfiles = QUALITY_PROFILE_OPTIONS.filter((profile) => profile.mode === 'batch');

  return (
    <div className="space-y-1">
      <label htmlFor={id} className="flex items-center gap-2 text-sm font-medium text-gray-700">
        <Gauge className="h-4 w-4 text-blue-600" />
        Processing profile
      </label>
      <Select
        value={value}
        onValueChange={(nextValue) => onValueChange(nextValue as TranscriptionQualityProfile)}
        disabled={disabled}
      >
        <SelectTrigger id={id} className="w-full" aria-label="Transcription processing profile">
          <SelectValue placeholder="Select processing profile" />
        </SelectTrigger>
        <SelectContent>
          {batchProfiles.map((profile) => (
            <SelectItem key={profile.id} value={profile.id}>
              {profile.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <p className="text-xs text-muted-foreground">{selected.description}</p>
      <p className="text-xs text-muted-foreground">
        VAD pause bridge: {selected.vadRedemptionMs} ms · Segment cap: {selected.maxSegmentSeconds} s
      </p>
    </div>
  );
}
