// CARIT (Cued-Action Response Inhibition Task) Types

export type TrialOutcome = 'Hit' | 'Miss' | 'FalseAlarm' | 'CorrectRejection';

export interface CaritTrialRecord {
  trialIndex: number;
  stim: string;
  shapeName: string;
  corrAns: 'go' | 'nogo';
  prepotency: number | null;
  isiSec: number;
  userPressed: boolean;
  reactionTimeMs: number | null; // relative to shape onset (0-800ms)
  outcome: TrialOutcome;
  timestampMs: number;
}

export interface PrepotencyMetric {
  prepotency: number; // 2, 3, or 4 Go trials before NoGo
  totalNogo: number;
  correctRejections: number;
  falseAlarms: number;
  inhibitionAccuracy: number; // 0.0 - 1.0
}

export interface CaritSessionSummary {
  id?: number;
  date: string; // YYYY-MM-DD
  timestamp: string;
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  mode: 'daily' | 'practice' | 'full_scan';
  totalTrials: number;
  hitCount: number;
  missCount: number;
  falseAlarmCount: number;
  corrRejectCount: number;
  totalAccuracy: number; // 0.0 - 1.0
  goAccuracy: number; // 0.0 - 1.0
  nogoAccuracy: number; // 0.0 - 1.0 (Inhibitory Control)
  meanRtMs: number; // Mean Reaction Time for Hits (ms)
  medianRtMs: number;
  rtStdDevMs: number; // Reaction Time Variability
  dPrime: number; // Signal Detection Sensitivity index
  cognitiveScore: number; // 0 - 100 overall cognitive readiness score
  xpAwarded: number;
  goldAwarded: number;
  prepotencyStats?: PrepotencyMetric[];
  trialDataJson?: string;
}

export interface TodayCaritSummary {
  hasCompletedToday: boolean;
  latestScore: number | null;
  latestMeanRtMs: number | null;
  latestInhibitionAcc: number | null;
  sessionsCountToday: number;
  latestSession: CaritSessionSummary | null;
}
