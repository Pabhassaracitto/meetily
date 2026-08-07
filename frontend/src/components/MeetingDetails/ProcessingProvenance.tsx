"use client";

import { useEffect, useState } from 'react';
import { Cpu, Gauge, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import {
  formatProcessingDuration,
  ProcessingRun,
} from '@/lib/processing-runs';

interface ProcessingProvenanceProps {
  meetingId: string;
}

const SOURCE_LABELS: Record<ProcessingRun['source_kind'], string> = {
  live: 'Live capture',
  import: 'Imported audio',
  retranscription: 'Retranscribed',
  recovery: 'Recovered transcript',
};

/**
 * Shows the latest non-content ASR provenance for a session. This makes model
 * and VAD choices reviewable without exposing transcript/audio text in the UI.
 */
export function ProcessingProvenance({ meetingId }: ProcessingProvenanceProps) {
  const [latestRun, setLatestRun] = useState<ProcessingRun | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    setLatestRun(null);

    invoke<ProcessingRun[]>('api_get_processing_runs', { meetingId })
      .then((runs) => {
        if (!cancelled) setLatestRun(runs[0] ?? null);
      })
      .catch((error) => {
        // Provenance is supplementary; keep the detail page usable if an old
        // database has not yet migrated or the lookup fails.
        console.warn('Failed to load processing provenance:', error);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [meetingId]);

  if (isLoading) {
    return (
      <div className="mb-3 flex items-center justify-center gap-1 text-xs text-muted-foreground">
        <Loader2 className="h-3 w-3 animate-spin" />
        Loading processing details
      </div>
    );
  }

  if (!latestRun) return null;

  const duration = formatProcessingDuration(latestRun.processing_time_ms);
  const source = SOURCE_LABELS[latestRun.source_kind] ?? latestRun.source_kind;
  const language = latestRun.language_hint ? ` · ${latestRun.language_hint}` : '';
  const vad = latestRun.vad_engine ? ` · VAD: ${latestRun.vad_engine}` : '';

  return (
    <div
      className="mb-3 flex flex-wrap items-center justify-center gap-x-2 gap-y-1 text-xs text-muted-foreground"
      title={`Processing run ${latestRun.id}`}
    >
      <span className="inline-flex items-center gap-1">
        <Cpu className="h-3.5 w-3.5" />
        {latestRun.provider}/{latestRun.model_id}
      </span>
      <span>{source}{language}{vad}</span>
      {duration && (
        <span className="inline-flex items-center gap-1">
          <Gauge className="h-3.5 w-3.5" />
          {duration}
        </span>
      )}
    </div>
  );
}
