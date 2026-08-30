// High-Precision CARIT Cognitive Task Game Engine (HCP Standard)
import type { CaritTrialDef } from '$lib/data/caritConditions';
import type { CaritTrialRecord, CaritSessionSummary, PrepotencyMetric } from '$lib/types/carit';

export const SHAPE_DURATION_MS = 600; // Shape display duration
export const RESPONSE_WINDOW_MS = 800; // Total window for valid response (600ms shape + 200ms shoulder)

export type EngineState = 
  | 'IDLE' 
  | 'COUNTDOWN' 
  | 'SHAPE' 
  | 'SHOULDER' 
  | 'ISI' 
  | 'FINISHED';

export interface EngineCallbacks {
  onStateChange: (state: EngineState) => void;
  onTrialStart: (trialIndex: number, trial: CaritTrialDef) => void;
  onTrialOutcome: (record: CaritTrialRecord) => void;
  onCountdownTick: (count: number) => void;
  onFinished: (summary: CaritSessionSummary, records: CaritTrialRecord[]) => void;
}

export class CaritEngine {
  private conditions: CaritTrialDef[] = [];
  private callbacks: EngineCallbacks;
  private mode: 'daily' | 'practice' | 'full_scan' = 'daily';

  private currentTrialIndex = -1;
  private state: EngineState = 'IDLE';
  private shapeOnsetTimestamp = 0;
  private hasRespondedThisTrial = false;
  private trialReactionTime: number | null = null;
  private timerId: any = null;
  private countdownTimerId: any = null;

  private records: CaritTrialRecord[] = [];

  constructor(callbacks: EngineCallbacks) {
    this.callbacks = callbacks;
  }

  public start(conditions: CaritTrialDef[], mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    this.stop();
    this.conditions = conditions;
    this.mode = mode;
    this.records = [];
    this.currentTrialIndex = -1;

    // Start 3-second countdown
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

  // Called when the user presses `<Space>` or taps the mobile action surface
  public handleUserAction() {
    if (this.state !== 'SHAPE' && this.state !== 'SHOULDER') {
      return; // Outside the valid response window
    }

    if (this.hasRespondedThisTrial) {
      return; // Already registered response for this trial
    }

    const now = performance.now();
    const rt = Math.round(now - this.shapeOnsetTimestamp);

    if (rt >= 0 && rt <= RESPONSE_WINDOW_MS) {
      this.hasRespondedThisTrial = true;
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
    this.trialReactionTime = null;

    // Phase 1: Show Shape (600ms)
    this.setState('SHAPE');
    this.shapeOnsetTimestamp = performance.now();
    this.callbacks.onTrialStart(this.currentTrialIndex, trial);

    this.timerId = setTimeout(() => {
      // Phase 2: Shoulder Window (200ms) into Fixation
      this.setState('SHOULDER');

      this.timerId = setTimeout(() => {
        // Evaluate Trial Outcome at end of 800ms response window
        this.evaluateTrial(trial);

        // Phase 3: Remainder of ISI Fixation
        this.setState('ISI');
        const remainingIsiMs = Math.max(100, Math.round(trial.isiSec * 1000) - 200);

        this.timerId = setTimeout(() => {
          this.nextTrial();
        }, remainingIsiMs);
      }, RESPONSE_WINDOW_MS - SHAPE_DURATION_MS); // 200ms
    }, SHAPE_DURATION_MS); // 600ms
  }

  private evaluateTrial(trial: CaritTrialDef) {
    const isGo = trial.corrAns === 'go';
    const pressed = this.hasRespondedThisTrial;
    const rt = this.trialReactionTime;

    let outcome: CaritTrialRecord['outcome'];
    if (isGo) {
      outcome = pressed ? 'Hit' : 'Miss';
    } else {
      outcome = pressed ? 'FalseAlarm' : 'CorrectRejection';
    }

    const record: CaritTrialRecord = {
      trialIndex: this.currentTrialIndex,
      stim: trial.stim,
      shapeName: trial.shapeName,
      corrAns: trial.corrAns,
      prepotency: trial.prepotency,
      isiSec: trial.isiSec,
      userPressed: pressed,
      reactionTimeMs: pressed ? rt : null,
      outcome,
      timestampMs: Date.now(),
    };

    this.records.push(record);
    this.callbacks.onTrialOutcome(record);
  }

  private finishSession() {
    this.setState('FINISHED');
    const summary = this.computeSummary();
    this.callbacks.onFinished(summary, this.records);
  }

  private setState(newState: EngineState) {
    this.state = newState;
    this.callbacks.onStateChange(newState);
  }

  private computeSummary(): CaritSessionSummary {
    const totalTrials = this.records.length;
    let hitCount = 0;
    let missCount = 0;
    let falseAlarmCount = 0;
    let corrRejectCount = 0;

    const hitReactionTimes: number[] = [];

    // Prepotency breakdown (2, 3, 4 Go trials before NoGo)
    const prepotencyMap = new Map<number, { total: number; cr: number; fa: number }>();
    [2, 3, 4].forEach(p => prepotencyMap.set(p, { total: 0, cr: 0, fa: 0 }));

    for (const r of this.records) {
      if (r.outcome === 'Hit') {
        hitCount++;
        if (r.reactionTimeMs !== null) hitReactionTimes.push(r.reactionTimeMs);
      } else if (r.outcome === 'Miss') {
        missCount++;
      } else if (r.outcome === 'FalseAlarm') {
        falseAlarmCount++;
        if (r.prepotency !== null && prepotencyMap.has(r.prepotency)) {
          const entry = prepotencyMap.get(r.prepotency)!;
          entry.total++;
          entry.fa++;
        }
      } else if (r.outcome === 'CorrectRejection') {
        corrRejectCount++;
        if (r.prepotency !== null && prepotencyMap.has(r.prepotency)) {
          const entry = prepotencyMap.get(r.prepotency)!;
          entry.total++;
          entry.cr++;
        }
      }
    }

    const totalGoTrials = hitCount + missCount;
    const totalNogoTrials = falseAlarmCount + corrRejectCount;

    const totalAccuracy = totalTrials > 0 ? (hitCount + corrRejectCount) / totalTrials : 0;
    const goAccuracy = totalGoTrials > 0 ? hitCount / totalGoTrials : 0;
    const nogoAccuracy = totalNogoTrials > 0 ? corrRejectCount / totalNogoTrials : 0;

    // Mean RT
    const meanRtMs = hitReactionTimes.length > 0
      ? Math.round(hitReactionTimes.reduce((a, b) => a + b, 0) / hitReactionTimes.length)
      : 0;

    // Median RT
    let medianRtMs = 0;
    if (hitReactionTimes.length > 0) {
      const sorted = [...hitReactionTimes].sort((a, b) => a - b);
      const mid = Math.floor(sorted.length / 2);
      medianRtMs = sorted.length % 2 !== 0 ? sorted[mid] : Math.round((sorted[mid - 1] + sorted[mid]) / 2);
    }

    // RT Standard Deviation (Variability)
    let rtStdDevMs = 0;
    if (hitReactionTimes.length > 1) {
      const variance = hitReactionTimes.reduce((acc, val) => acc + Math.pow(val - meanRtMs, 2), 0) / (hitReactionTimes.length - 1);
      rtStdDevMs = Math.round(Math.sqrt(variance));
    }

    // Signal Detection d-prime with log-linear correction (Hautus, 1995)
    // Hit Rate adjusted = (Hits + 0.5) / (GoTrials + 1)
    // False Alarm Rate adjusted = (FalseAlarms + 0.5) / (NoGoTrials + 1)
    const adjHitRate = (hitCount + 0.5) / (totalGoTrials + 1);
    const adjFaRate = (falseAlarmCount + 0.5) / (totalNogoTrials + 1);
    const zHit = normalInv(adjHitRate);
    const zFa = normalInv(adjFaRate);
    const dPrime = parseFloat((zHit - zFa).toFixed(2));

    // Cognitive Readiness Score (0 - 100):
    // 40% Inhibitory Accuracy (NoGo), 30% Go Accuracy, 30% Speed Efficiency (relative to 300-600ms benchmark)
    const speedScore = meanRtMs > 0 ? Math.max(0, Math.min(100, Math.round(100 - ((meanRtMs - 280) / 4.0)))) : 50;
    const cognitiveScore = Math.max(1, Math.min(100, Math.round(
      (nogoAccuracy * 40) + (goAccuracy * 30) + (speedScore * 0.3)
    )));

    // Prepotency stats
    const prepotencyStats: PrepotencyMetric[] = [];
    prepotencyMap.forEach((entry, prepotency) => {
      if (entry.total > 0) {
        prepotencyStats.push({
          prepotency,
          totalNogo: entry.total,
          correctRejections: entry.cr,
          falseAlarms: entry.fa,
          inhibitionAccuracy: parseFloat((entry.cr / entry.total).toFixed(2)),
        });
      }
    });

    // Goblin XP & Gold calculation
    const xpAwarded = Math.round(cognitiveScore * 1.5 + (this.mode === 'full_scan' ? 80 : 35));
    const goldAwarded = Math.round(cognitiveScore * 0.4 + (nogoAccuracy > 0.9 ? 15 : 5));

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
      hitCount,
      missCount,
      falseAlarmCount,
      corrRejectCount,
      totalAccuracy: parseFloat(totalAccuracy.toFixed(3)),
      goAccuracy: parseFloat(goAccuracy.toFixed(3)),
      nogoAccuracy: parseFloat(nogoAccuracy.toFixed(3)),
      meanRtMs,
      medianRtMs,
      rtStdDevMs,
      dPrime,
      cognitiveScore,
      xpAwarded,
      goldAwarded,
      prepotencyStats,
      trialDataJson: JSON.stringify(this.records),
    };
  }
}

// Probit / Inverse Standard Normal CDF approximation (Winitzki approximation)
function normalInv(p: number): number {
  if (p <= 0) return -3.5;
  if (p >= 1) return 3.5;
  if (p === 0.5) return 0;

  // Abramowitz and Stegun rational approximation
  const a1 = -39.69683028665376;
  const a2 = 220.9460984245205;
  const a3 = -275.9285104469687;
  const a4 = 138.3577518672690;
  const a5 = -30.66479806614716;
  const a6 = 2.506628277459239;

  const b1 = -54.47609879822406;
  const b2 = 161.5858368580409;
  const b3 = -155.6989798598866;
  const b4 = 66.80131188771972;
  const b5 = -13.28068155288572;

  const c1 = -0.007784894002430293;
  const c2 = -0.3223964580411365;
  const c3 = -2.400758277161838;
  const c4 = -2.549732539343734;
  const c5 = 4.374664141464968;
  const c6 = 2.938163982698783;

  const d1 = 0.007784695709041462;
  const d2 = 0.3224671290700398;
  const d3 = 2.445134137142996;
  const d4 = 3.754408661907416;

  const p_low = 0.02425;
  const p_high = 1 - p_low;

  let q: number, r: number;
  if (p < p_low) {
    q = Math.sqrt(-2 * Math.log(p));
    return (((((c1 * q + c2) * q + c3) * q + c4) * q + c5) * q + c6) /
           ((((d1 * q + d2) * q + d3) * q + d4) * q + 1);
  }
  if (p <= p_high) {
    q = p - 0.5;
    r = q * q;
    return (((((a1 * r + a2) * r + a3) * r + a4) * r + a5) * r + a6) * q /
           (((((b1 * r + b2) * r + b3) * r + b4) * r + b5) * r + 1);
  }
  q = Math.sqrt(-2 * Math.log(1 - p));
  return -(((((c1 * q + c2) * q + c3) * q + c4) * q + c5) * q + c6) /
          ((((d1 * q + d2) * q + d3) * q + d4) * q + 1);
}
