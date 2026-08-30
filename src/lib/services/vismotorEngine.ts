// HCP VISMOTOR High-Precision Choice Reaction Game Engine
import type { VismotorTrialDef, TargetSide } from '$lib/data/vismotorConditions';
import type { VismotorTrialRecord, VismotorSessionSummary } from '$lib/types/vismotor';

export type VismotorEngineState = 
  | 'IDLE' 
  | 'COUNTDOWN' 
  | 'FIXATION' 
  | 'TARGET' 
  | 'FINISHED';

export interface VismotorEngineCallbacks {
  onStateChange: (state: VismotorEngineState) => void;
  onCountdownTick: (count: number) => void;
  onTrialStart: (trialIndex: number, trial: VismotorTrialDef) => void;
  onTrialEnd: (record: VismotorTrialRecord) => void;
  onFinished: (summary: VismotorSessionSummary) => void;
}

export class VismotorEngine {
  private conditions: VismotorTrialDef[] = [];
  private callbacks: VismotorEngineCallbacks;
  private mode: 'daily' | 'practice' | 'full_scan' = 'daily';

  private currentTrialIndex = -1;
  private state: VismotorEngineState = 'IDLE';
  private targetOnsetTimestamp = 0;
  private hasRespondedThisTrial = false;
  private userSidePressed: TargetSide | null = null;
  private trialReactionTime: number | null = null;

  private timerId: any = null;
  private countdownTimerId: any = null;
  private records: VismotorTrialRecord[] = [];

  constructor(callbacks: VismotorEngineCallbacks) {
    this.callbacks = callbacks;
  }

  public start(conditions: VismotorTrialDef[], mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    this.stop();
    this.conditions = conditions;
    this.mode = mode;
    this.records = [];
    this.currentTrialIndex = -1;

    // 3-second countdown
    this.setState('COUNTDOWN');
    let count = 3;
    this.callbacks.onCountdownTick(count);

    this.countdownTimerId = setInterval(() => {
      count--;
      if (count > 0) {
        this.callbacks.onCountdownTick(count);
      } else {
        clearInterval(this.countdownTimerId);
        this.countdownTimerId = null;
        this.nextTrial();
      }
    }, 1000);
  }

  public stop() {
    if (this.timerId) clearTimeout(this.timerId);
    if (this.countdownTimerId) clearInterval(this.countdownTimerId);
    this.timerId = null;
    this.countdownTimerId = null;
    this.setState('IDLE');
  }

  public handleUserAction(side: TargetSide) {
    if (this.state !== 'TARGET' && this.state !== 'FIXATION') return;
    if (this.hasRespondedThisTrial) return;

    const now = performance.now();
    const rt = Math.round(now - this.targetOnsetTimestamp);

    // Valid response window up to 800ms from target onset
    if (rt >= 0 && rt <= 1000) {
      this.hasRespondedThisTrial = true;
      this.userSidePressed = side;
      this.trialReactionTime = rt;
    }
  }

  private nextTrial() {
    this.currentTrialIndex++;

    if (this.currentTrialIndex >= this.conditions.length) {
      this.finishSession();
      return;
    }

    const trial = this.conditions[this.currentTrialIndex];
    this.hasRespondedThisTrial = false;
    this.userSidePressed = null;
    this.trialReactionTime = null;

    // Phase 1: Target Display (500ms)
    this.setState('TARGET');
    this.targetOnsetTimestamp = performance.now();
    this.callbacks.onTrialStart(this.currentTrialIndex, trial);

    this.timerId = setTimeout(() => {
      // Phase 2: Fixation ISI (1.5s - 3.0s)
      this.setState('FIXATION');

      this.timerId = setTimeout(() => {
        // Evaluate Trial
        const isCorrect = this.userSidePressed === trial.side;
        const record: VismotorTrialRecord = {
          trialIndex: this.currentTrialIndex,
          side: trial.side,
          userSidePressed: this.userSidePressed,
          isCorrect,
          reactionTimeMs: this.trialReactionTime,
          timestampMs: Date.now(),
        };

        this.records.push(record);
        this.callbacks.onTrialEnd(record);

        this.nextTrial();
      }, Math.max(500, Math.round(trial.fixationDurationSec * 1000) - 500));
    }, trial.targetDurationMs);
  }

  private finishSession() {
    this.setState('FINISHED');
    const summary = this.computeSummary();
    this.callbacks.onFinished(summary);
  }

  private setState(newState: VismotorEngineState) {
    this.state = newState;
    this.callbacks.onStateChange(newState);
  }

  private computeSummary(): VismotorSessionSummary {
    const totalTrials = this.records.length;
    const correctCount = this.records.filter(r => r.isCorrect).length;
    const incorrectCount = totalTrials - correctCount;
    const accuracy = totalTrials > 0 ? correctCount / totalTrials : 0;

    const allCorrectRts = this.records
      .filter(r => r.isCorrect && r.reactionTimeMs !== null)
      .map(r => r.reactionTimeMs!);

    const leftCorrectRts = this.records
      .filter(r => r.isCorrect && r.side === 'left' && r.reactionTimeMs !== null)
      .map(r => r.reactionTimeMs!);

    const rightCorrectRts = this.records
      .filter(r => r.isCorrect && r.side === 'right' && r.reactionTimeMs !== null)
      .map(r => r.reactionTimeMs!);

    const meanRtMs = allCorrectRts.length > 0
      ? Math.round(allCorrectRts.reduce((a, b) => a + b, 0) / allCorrectRts.length)
      : 0;

    const meanLeftRtMs = leftCorrectRts.length > 0
      ? Math.round(leftCorrectRts.reduce((a, b) => a + b, 0) / leftCorrectRts.length)
      : 0;

    const meanRightRtMs = rightCorrectRts.length > 0
      ? Math.round(rightCorrectRts.reduce((a, b) => a + b, 0) / rightCorrectRts.length)
      : 0;

    const hemisphericDifferenceMs = Math.abs(meanLeftRtMs - meanRightRtMs);

    // RT Standard Deviation
    let rtStdDevMs = 0;
    if (allCorrectRts.length > 1) {
      const variance = allCorrectRts.reduce((acc, val) => acc + Math.pow(val - meanRtMs, 2), 0) / (allCorrectRts.length - 1);
      rtStdDevMs = Math.round(Math.sqrt(variance));
    }

    // Visuomotor Score (0 - 100):
    // 50% Accuracy, 35% Speed Efficiency (250 - 550ms), 15% Motor Symmetry (<50ms difference is optimal)
    const speedScore = meanRtMs > 0 ? Math.max(0, Math.min(100, Math.round(100 - ((meanRtMs - 260) / 3.0)))) : 50;
    const symmetryScore = Math.max(0, Math.min(100, Math.round(100 - (hemisphericDifferenceMs * 1.5))));

    const vismotorScore = Math.max(1, Math.min(100, Math.round(
      (accuracy * 50) + (speedScore * 0.35) + (symmetryScore * 0.15)
    )));

    const xpAwarded = Math.round(vismotorScore * 1.5 + 30);
    const goldAwarded = Math.round(vismotorScore * 0.4 + (accuracy >= 0.95 ? 12 : 5));

    const now = new Date();
    const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    const hour = now.getHours();
    const timeOfDay = hour < 12 ? 'morning' : hour < 17 ? 'afternoon' : hour < 21 ? 'evening' : 'night';

    return {
      date: dateStr,
      timestamp: now.toISOString(),
      timeOfDay,
      mode: this.mode,
      totalTrials,
      correctCount,
      incorrectCount,
      accuracy: parseFloat(accuracy.toFixed(3)),
      meanRtMs,
      meanLeftRtMs,
      meanRightRtMs,
      hemisphericDifferenceMs,
      rtStdDevMs,
      vismotorScore,
      xpAwarded,
      goldAwarded,
      trialDataJson: JSON.stringify(this.records),
    };
  }
}
