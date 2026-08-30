// HCP FACENAME Types

export interface MemorizeTrialRecord {
  trialIndex: number;
  faceId: string;
  name: string;
  durationMs: number;
}

export interface FaceDistractorRecord {
  trialIndex: number;
  stim: string;
  corrAns: 'go' | 'nogo';
  userPressed: boolean;
  rtMs: number | null;
  isCorrect: boolean;
}

export interface RecallTrialRecord {
  trialIndex: number;
  faceId: string;
  correctName: string;
  selectedName: string | null;
  options: string[];
  isCorrect: boolean;
  rtMs: number | null; // Retrieval latency (ms)
}

export interface FaceNameSessionSummary {
  id?: number;
  date: string; // YYYY-MM-DD
  timestamp: string;
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  mode: 'daily' | 'practice' | 'full_scan';
  totalMemorized: number;
  totalRecalled: number;
  correctRecallCount: number;
  recallAccuracy: number; // 0.0 - 1.0 (Associative Memory Retention)
  meanRecallRtMs: number; // Retrieval Latency (ms)
  medianRecallRtMs: number;
  distractorAccuracy: number; // 0.0 - 1.0 (Inhibitory control during delay)
  memoryScore: number; // 0 - 100 Overall Memory Readiness Score
  xpAwarded: number;
  goldAwarded: number;
  trialDataJson?: string;
}

export interface TodayFaceNameSummary {
  hasCompletedToday: boolean;
  latestScore: number | null;
  latestRecallAccuracy: number | null;
  latestRecallRtMs: number | null;
  sessionsCountToday: number;
  latestSession: FaceNameSessionSummary | null;
}
