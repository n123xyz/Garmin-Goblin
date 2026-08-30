// HCP FACENAME High-Precision Memory Game Engine
import { 
  PRACTICE_FACES, 
  PRACTICE_DISTRACTORS,
  DAILY_MALE_FACES, 
  DAILY_FEMALE_FACES, 
  DAILY_DISTRACTOR_SET_1, 
  DAILY_DISTRACTOR_SET_2,
  generateRecallOptions,
  type FaceItem, 
  type DistractorTrialDef 
} from '$lib/data/facenameConditions';
import type { 
  RecallTrialRecord, 
  FaceDistractorRecord, 
  FaceNameSessionSummary 
} from '$lib/types/facename';

export type FaceEngineState = 
  | 'IDLE' 
  | 'COUNTDOWN' 
  | 'CUE' 
  | 'MEMORIZE' 
  | 'DISTRACTOR_SHAPE' 
  | 'DISTRACTOR_ISI' 
  | 'RECALL' 
  | 'FINISHED';

export interface FaceEngineCallbacks {
  onStateChange: (state: FaceEngineState) => void;
  onCountdownTick: (count: number) => void;
  onCueChange: (cueTitle: string, cueSubtitle: string) => void;
  onMemorizeFace: (face: FaceItem, index: number, total: number) => void;
  onDistractorStart: (trial: DistractorTrialDef, index: number, total: number) => void;
  onRecallFace: (face: FaceItem, options: string[], index: number, total: number) => void;
  onRecallFeedback?: (isCorrect: boolean, correctName: string, rtMs: number) => void;
  onFinished: (summary: FaceNameSessionSummary) => void;
}

export class FaceNameEngine {
  private callbacks: FaceEngineCallbacks;
  private mode: 'daily' | 'practice' | 'full_scan' = 'daily';
  private state: FaceEngineState = 'IDLE';

  // Blocks setup
  private blocks: Array<{
    type: 'MEMORIZE' | 'DISTRACTOR' | 'RECALL';
    faces?: FaceItem[];
    distractors?: DistractorTrialDef[];
  }> = [];

  private currentBlockIdx = 0;
  private currentItemIdx = 0;
  private timerId: any = null;
  private countdownTimerId: any = null;

  // Recall timing
  private recallOnsetTimestamp = 0;
  private hasAnsweredRecall = false;

  // Distractor timing
  private distractorOnsetTimestamp = 0;
  private hasAnsweredDistractor = false;
  private distractorUserPressed = false;
  private distractorRt: number | null = null;

  // Data collection
  private recallRecords: RecallTrialRecord[] = [];
  private distractorRecords: FaceDistractorRecord[] = [];
  private totalMemorizedCount = 0;

  constructor(callbacks: FaceEngineCallbacks) {
    this.callbacks = callbacks;
  }

  public start(mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    this.stop();
    this.mode = mode;
    this.recallRecords = [];
    this.distractorRecords = [];
    this.currentBlockIdx = 0;
    this.currentItemIdx = 0;
    this.totalMemorizedCount = 0;

    if (mode === 'practice') {
      this.totalMemorizedCount = PRACTICE_FACES.length;
      this.blocks = [
        { type: 'MEMORIZE', faces: [...PRACTICE_FACES] },
        { type: 'DISTRACTOR', distractors: [...PRACTICE_DISTRACTORS] },
        { type: 'RECALL', faces: [...PRACTICE_FACES].sort(() => Math.random() - 0.5) },
      ];
    } else {
      // Daily Mode: 2 blocks (Males then Females)
      this.totalMemorizedCount = DAILY_MALE_FACES.length + DAILY_FEMALE_FACES.length;
      this.blocks = [
        { type: 'MEMORIZE', faces: [...DAILY_MALE_FACES] },
        { type: 'DISTRACTOR', distractors: [...DAILY_DISTRACTOR_SET_1] },
        { type: 'RECALL', faces: [...DAILY_MALE_FACES].sort(() => Math.random() - 0.5) },
        { type: 'MEMORIZE', faces: [...DAILY_FEMALE_FACES] },
        { type: 'DISTRACTOR', distractors: [...DAILY_DISTRACTOR_SET_2] },
        { type: 'RECALL', faces: [...DAILY_FEMALE_FACES].sort(() => Math.random() - 0.5) },
      ];
    }

    // Start 3s countdown
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
        this.startNextBlock();
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

  // Handle Distractor Tap (Go / NoGo press)
  public handleDistractorAction() {
    if (this.state !== 'DISTRACTOR_SHAPE' && this.state !== 'DISTRACTOR_ISI') return;
    if (this.hasAnsweredDistractor) return;

    this.hasAnsweredDistractor = true;
    this.distractorUserPressed = true;
    this.distractorRt = Math.round(performance.now() - this.distractorOnsetTimestamp);
  }

  // Handle Recall Multiple Choice Selection
  public handleRecallSelection(chosenName: string) {
    if (this.state !== 'RECALL' || this.hasAnsweredRecall) return;
    this.hasAnsweredRecall = true;

    const block = this.blocks[this.currentBlockIdx];
    if (!block.faces) return;
    const face = block.faces[this.currentItemIdx];
    const rt = Math.round(performance.now() - this.recallOnsetTimestamp);
    const isCorrect = chosenName === face.name;

    this.recallRecords.push({
      trialIndex: this.recallRecords.length,
      faceId: face.id,
      correctName: face.name,
      selectedName: chosenName,
      options: [],
      isCorrect,
      rtMs: rt,
    });

    if (this.callbacks.onRecallFeedback) {
      this.callbacks.onRecallFeedback(isCorrect, face.name, rt);
    }

    if (this.timerId) clearTimeout(this.timerId);

    // Transition quickly to next face
    this.timerId = setTimeout(() => {
      this.nextItemInBlock();
    }, 500);
  }

  private startNextBlock() {
    if (this.currentBlockIdx >= this.blocks.length) {
      this.finishSession();
      return;
    }

    const block = this.blocks[this.currentBlockIdx];
    this.currentItemIdx = 0;

    // Show 2-second Block Cue
    this.setState('CUE');
    if (block.type === 'MEMORIZE') {
      this.callbacks.onCueChange('MEMORIZE FACES & NAMES', 'Study each face and remember their name carefully (4s per face)');
    } else if (block.type === 'DISTRACTOR') {
      this.callbacks.onCueChange('DISTRACTION SPEED TASK', 'Tap on Polygon Go shapes! Withhold on Circles and Squares!');
    } else if (block.type === 'RECALL') {
      this.callbacks.onCueChange('NAME RECALL & RECOGNITION', 'Select the correct name for each face as fast and accurately as possible');
    }

    this.timerId = setTimeout(() => {
      this.startCurrentItem();
    }, 2000);
  }

  private startCurrentItem() {
    const block = this.blocks[this.currentBlockIdx];

    if (block.type === 'MEMORIZE') {
      if (!block.faces || this.currentItemIdx >= block.faces.length) {
        this.finishCurrentBlock();
        return;
      }
      const face = block.faces[this.currentItemIdx];
      this.setState('MEMORIZE');
      this.callbacks.onMemorizeFace(face, this.currentItemIdx + 1, block.faces.length);

      this.timerId = setTimeout(() => {
        this.nextItemInBlock();
      }, 4000); // 4 seconds per face

    } else if (block.type === 'DISTRACTOR') {
      if (!block.distractors || this.currentItemIdx >= block.distractors.length) {
        this.finishCurrentBlock();
        return;
      }
      const dist = block.distractors[this.currentItemIdx];
      this.hasAnsweredDistractor = false;
      this.distractorUserPressed = false;
      this.distractorRt = null;
      this.distractorOnsetTimestamp = performance.now();

      // Shape presentation (600ms)
      this.setState('DISTRACTOR_SHAPE');
      this.callbacks.onDistractorStart(dist, this.currentItemIdx + 1, block.distractors.length);

      this.timerId = setTimeout(() => {
        this.setState('DISTRACTOR_ISI');

        this.timerId = setTimeout(() => {
          // Evaluate Distractor trial
          const isGo = dist.corrAns === 'go';
          const isCorrect = (isGo && this.distractorUserPressed) || (!isGo && !this.distractorUserPressed);
          this.distractorRecords.push({
            trialIndex: this.distractorRecords.length,
            stim: dist.stim,
            corrAns: dist.corrAns,
            userPressed: this.distractorUserPressed,
            rtMs: this.distractorRt,
            isCorrect,
          });

          this.nextItemInBlock();
        }, Math.max(800, Math.round(dist.isiSec * 1000) - 600));
      }, 600);

    } else if (block.type === 'RECALL') {
      if (!block.faces || this.currentItemIdx >= block.faces.length) {
        this.finishCurrentBlock();
        return;
      }
      const face = block.faces[this.currentItemIdx];
      this.hasAnsweredRecall = false;
      this.recallOnsetTimestamp = performance.now();

      const options = generateRecallOptions(face, block.faces);
      this.setState('RECALL');
      this.callbacks.onRecallFace(face, options, this.currentItemIdx + 1, block.faces.length);

      // 6-second timeout if no answer
      this.timerId = setTimeout(() => {
        if (!this.hasAnsweredRecall) {
          this.handleRecallSelection(''); // Miss / Timeout
        }
      }, 6000);
    }
  }

  private nextItemInBlock() {
    this.currentItemIdx++;
    const block = this.blocks[this.currentBlockIdx];
    const maxItems = block.type === 'DISTRACTOR' 
      ? (block.distractors?.length || 0) 
      : (block.faces?.length || 0);

    if (this.currentItemIdx >= maxItems) {
      this.finishCurrentBlock();
    } else {
      this.startCurrentItem();
    }
  }

  private finishCurrentBlock() {
    this.currentBlockIdx++;
    this.startNextBlock();
  }

  private finishSession() {
    this.setState('FINISHED');
    const summary = this.computeSummary();
    this.callbacks.onFinished(summary);
  }

  private setState(newState: FaceEngineState) {
    this.state = newState;
    this.callbacks.onStateChange(newState);
  }

  private computeSummary(): FaceNameSessionSummary {
    const totalRecalled = this.recallRecords.length;
    const correctRecallCount = this.recallRecords.filter(r => r.isCorrect).length;
    const recallAccuracy = totalRecalled > 0 ? correctRecallCount / totalRecalled : 0;

    const correctRts = this.recallRecords
      .filter(r => r.isCorrect && r.rtMs !== null)
      .map(r => r.rtMs!);

    const meanRecallRtMs = correctRts.length > 0
      ? Math.round(correctRts.reduce((a, b) => a + b, 0) / correctRts.length)
      : 0;

    let medianRecallRtMs = 0;
    if (correctRts.length > 0) {
      const sorted = [...correctRts].sort((a, b) => a - b);
      const mid = Math.floor(sorted.length / 2);
      medianRecallRtMs = sorted.length % 2 !== 0 ? sorted[mid] : Math.round((sorted[mid - 1] + sorted[mid]) / 2);
    }

    const distractorTotal = this.distractorRecords.length;
    const distractorCorrect = this.distractorRecords.filter(r => r.isCorrect).length;
    const distractorAccuracy = distractorTotal > 0 ? distractorCorrect / distractorTotal : 1.0;

    // Memory Readiness Score (0 - 100):
    // 60% Recall Accuracy, 20% Distractor Inhibition, 20% Retrieval Speed (Benchmark 1200 - 3000ms)
    const speedScore = meanRecallRtMs > 0 
      ? Math.max(0, Math.min(100, Math.round(100 - ((meanRecallRtMs - 1000) / 30.0))))
      : 50;

    const memoryScore = Math.max(1, Math.min(100, Math.round(
      (recallAccuracy * 60) + (distractorAccuracy * 20) + (speedScore * 0.2)
    )));

    // Goblin XP & Gold calculation
    const xpAwarded = Math.round(memoryScore * 1.6 + (this.mode === 'daily' ? 45 : 20));
    const goldAwarded = Math.round(memoryScore * 0.45 + (recallAccuracy >= 0.9 ? 15 : 5));

    const now = new Date();
    const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    const hour = now.getHours();
    const timeOfDay = hour < 12 ? 'morning' : hour < 17 ? 'afternoon' : hour < 21 ? 'evening' : 'night';

    return {
      date: dateStr,
      timestamp: now.toISOString(),
      timeOfDay,
      mode: this.mode,
      totalMemorized: this.totalMemorizedCount,
      totalRecalled,
      correctRecallCount,
      recallAccuracy: parseFloat(recallAccuracy.toFixed(3)),
      meanRecallRtMs,
      medianRecallRtMs,
      distractorAccuracy: parseFloat(distractorAccuracy.toFixed(3)),
      memoryScore,
      xpAwarded,
      goldAwarded,
      trialDataJson: JSON.stringify({ recall: this.recallRecords, distractor: this.distractorRecords }),
    };
  }
}
