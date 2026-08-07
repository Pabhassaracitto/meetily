/**
 * Storage Service
 *
 * Handles all meeting storage and retrieval Tauri backend calls (SQLite persistence).
 * Pure 1-to-1 wrapper - no error handling changes, exact same behavior as direct invoke calls.
 */

import { invoke } from '@tauri-apps/api/core';
import { Transcript } from '@/types';
import { SessionType } from '@/lib/session-types';
import {
  ProcessingSourceKind,
  TranscriptionRunMetadata,
} from '@/lib/processing-runs';

export interface SaveMeetingRequest {
  meetingTitle: string;
  transcripts: Transcript[];
  folderPath: string | null;
  sessionType: SessionType;
  sourceKind?: ProcessingSourceKind;
  processingMetadata?: TranscriptionRunMetadata;
}

export interface SaveMeetingResponse {
  meeting_id: string;
  processing_run_id?: string;
}

export interface Meeting {
  id: string;
  title: string;
  [key: string]: any; // Allow additional properties from backend
}

/**
 * Storage Service
 * Singleton service for managing meeting storage operations
 */
export class StorageService {
  /**
   * Save meeting transcript to SQLite database
   * @param meetingTitle - Title of the meeting
   * @param transcripts - Array of transcript segments
   * @param folderPath - Optional folder path for audio file
   * @param sessionType - The purpose of the captured session
   * @param processingMetadata - Non-content ASR provider/model/VAD provenance
   * @param sourceKind - Capture path that created the transcript
   * @returns Promise with the session and immutable processing-run IDs
   */
  async saveMeeting(
    meetingTitle: string,
    transcripts: Transcript[],
    folderPath: string | null,
    sessionType: SessionType = 'meeting',
    processingMetadata?: TranscriptionRunMetadata,
    sourceKind: ProcessingSourceKind = 'live'
  ): Promise<SaveMeetingResponse> {
    return invoke<SaveMeetingResponse>('api_save_transcript', {
      meetingTitle,
      transcripts,
      folderPath,
      sessionType,
      processingMetadata,
      sourceKind,
    });
  }

  /**
   * Get meeting details by ID
   * @param meetingId - ID of the meeting to fetch
   * @returns Promise with meeting details
   */
  async getMeeting(meetingId: string): Promise<Meeting> {
    return invoke<Meeting>('api_get_meeting', { meetingId });
  }

  /**
   * Get list of all meetings
   * @returns Promise with array of meetings
   */
  async getMeetings(): Promise<Meeting[]> {
    return invoke<Meeting[]>('api_get_meetings');
  }
}

// Export singleton instance
export const storageService = new StorageService();
