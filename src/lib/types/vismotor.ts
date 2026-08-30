// HCP VISMOTOR Types
import type { TargetSide } from '$lib/data/vismotorConditions';

export interface VismotorTrialRecord {
  trialIndex: number;
  side: TargetSide;
  userSidePressed: TargetSide | null;
  isCorrect: boolean;
  reactionTimeMs: number | null; // ms from target appearance
  timestampMs: number;
}

export interface VismotorSessionSummary {
  id?: number;
  date: string; // YYYY-MM-DD
  timestamp: string;
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  mode: 'daily' | 'practice' | 'full_scan';
  totalTrials: number;
  correctCount: number;
  incorrectCount: number;
  accuracy: number; // 0.0 - 1.0
  meanRtMs: number; // Overall mean RT
  meanLeftRtMs: number; // Left side mean RT
  meanRightRtMs: number; // Right side mean RT
  hemisphericDifferenceMs: number; // |Left RT - Right RT| (Motor symmetry)
  rtStdDevMs: number; // Motor consistency variability
  vismotorScore: number; // 0 - 100 Visuomotor Speed & Precision Score
  xpAwarded: number;
  goldAwarded: number;
  trialDataJson?: string;
}

export interface TodayVismotorSummary {
  hasCompletedToday: boolean;
  latestScore: number | null;
  latestMeanRtMs: number | null;
  latestAccuracy: number | null;
  sessionsCountToday: number;
  latestSession: VismotorSessionSummary | null;
}
