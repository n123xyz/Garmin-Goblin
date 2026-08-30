// HCP VISMOTOR (Visuomotor Choice Reaction Task) Condition Sequences

export type TargetSide = 'left' | 'right';

export interface VismotorTrialDef {
  trialIndex: number;
  side: TargetSide;
  targetImage: string; // e.g. 'leftSquares.png' or 'rightSquares.png'
  targetDurationMs: number; // exact 500ms
  fixationDurationSec: number; // 1.5s - 3.0s
}

// 1. PRACTICE CONDITIONS (8 trials, ~30s)
export const VISMOTOR_PRACTICE_CONDITIONS: VismotorTrialDef[] = [
  { trialIndex: 0, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
  { trialIndex: 1, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.5 },
  { trialIndex: 2, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 1.8 },
  { trialIndex: 3, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.2 },
  { trialIndex: 4, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
  { trialIndex: 5, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 1.8 },
  { trialIndex: 6, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.4 },
  { trialIndex: 7, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
];

// 2. DAILY TEST CONDITIONS (20 trials, ~60s balanced left & right)
export const VISMOTOR_DAILY_CONDITIONS: VismotorTrialDef[] = [
  { trialIndex: 0, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.2 },
  { trialIndex: 1, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 1.8 },
  { trialIndex: 2, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.5 },
  { trialIndex: 3, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
  { trialIndex: 4, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.4 },
  { trialIndex: 5, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 1.9 },
  { trialIndex: 6, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.6 },
  { trialIndex: 7, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.1 },
  { trialIndex: 8, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 1.8 },
  { trialIndex: 9, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.3 },
  { trialIndex: 10, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
  { trialIndex: 11, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.5 },
  { trialIndex: 12, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 1.9 },
  { trialIndex: 13, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.2 },
  { trialIndex: 14, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.4 },
  { trialIndex: 15, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 1.8 },
  { trialIndex: 16, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.1 },
  { trialIndex: 17, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.5 },
  { trialIndex: 18, side: 'left', targetImage: 'leftSquares.png', targetDurationMs: 500, fixationDurationSec: 2.0 },
  { trialIndex: 19, side: 'right', targetImage: 'rightSquares.png', targetDurationMs: 500, fixationDurationSec: 2.2 },
];
