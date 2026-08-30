// HCP FACENAME Condition Sequences & Definitions

export interface FaceItem {
  id: string;
  image: string; // relative to /facename/
  name: string;
  gender: 'M' | 'F';
}

export interface DistractorTrialDef {
  stim: string; // shape image e.g. 'plaque.png'
  shapeName: string;
  corrAns: 'go' | 'nogo';
  isiSec: number;
}

export interface FaceNameBlockDef {
  blockIndex: number;
  blockType: 'MEMORIZE' | 'DISTRACTOR' | 'RECALL';
  faces?: FaceItem[];
  distractorTrials?: DistractorTrialDef[];
  recallOptionsMap?: Record<string, string[]>; // face.id -> [options including correct name]
}

// 1. PRACTICE CONDITIONS (3 Face-Name pairs, ~45s)
export const PRACTICE_FACES: FaceItem[] = [
  { id: 'CF0058', image: 'CF0058_1100_NE.jpg', name: 'Julie', gender: 'F' },
  { id: 'HF1205', image: 'HF1205_1100_NE.jpg', name: 'Cindy', gender: 'F' },
  { id: 'CF0025', image: 'CF0025_1100_NE.jpg', name: 'Amy', gender: 'F' },
];

export const PRACTICE_DISTRACTORS: DistractorTrialDef[] = [
  { stim: 'plaque.png', shapeName: 'Plaque', corrAns: 'go', isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'Pentagon', corrAns: 'go', isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'Circle', corrAns: 'nogo', isiSec: 2.0 },
  { stim: 'hexagon.png', shapeName: 'Hexagon', corrAns: 'go', isiSec: 1.5 },
];

// 2. DAILY TEST CONDITIONS (2 Blocks: 5 Men pairs + Distractor + Recall; 5 Women pairs + Distractor + Recall)
export const DAILY_MALE_FACES: FaceItem[] = [
  { id: 'CM0001', image: 'CM0001_1100_NE.jpg', name: 'Jim', gender: 'M' },
  { id: 'BM0606', image: 'BM0606_1100_NE.jpg', name: 'William', gender: 'M' },
  { id: 'BM0608', image: 'BM0608_1100_NE.jpg', name: 'Richard', gender: 'M' },
  { id: 'CM0008', image: 'CM0008_1100_NE.jpg', name: 'Norman', gender: 'M' },
  { id: 'MM0903', image: 'MM0903_1100_NE.jpg', name: 'Adam', gender: 'M' },
];

export const DAILY_FEMALE_FACES: FaceItem[] = [
  { id: 'CF0003', image: 'CF0003_1100_NE.jpg', name: 'Carol', gender: 'F' },
  { id: 'BF0607', image: 'BF0607_1100_NE.jpg', name: 'Helen', gender: 'F' },
  { id: 'HF1208', image: 'HF1208_1100_NE.jpg', name: 'Angela', gender: 'F' },
  { id: 'BF0612', image: 'BF0612_1100_NE.jpg', name: 'Sharon', gender: 'F' },
  { id: 'CF0020', image: 'CF0020_1100_NE.jpg', name: 'Diane', gender: 'F' },
];

export const DAILY_DISTRACTOR_SET_1: DistractorTrialDef[] = [
  { stim: 'pentagon.png', shapeName: 'Pentagon', corrAns: 'go', isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'Circle', corrAns: 'nogo', isiSec: 2.0 },
  { stim: 'octagon.png', shapeName: 'Octagon', corrAns: 'go', isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'Square', corrAns: 'nogo', isiSec: 2.0 },
  { stim: 'parallelogram.png', shapeName: 'Parallelogram', corrAns: 'go', isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'Trapezoid', corrAns: 'go', isiSec: 1.5 },
];

export const DAILY_DISTRACTOR_SET_2: DistractorTrialDef[] = [
  { stim: 'hexagon.png', shapeName: 'Hexagon', corrAns: 'go', isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'Plaque', corrAns: 'go', isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'Square', corrAns: 'nogo', isiSec: 2.0 },
  { stim: 'pentagon.png', shapeName: 'Pentagon', corrAns: 'go', isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'Circle', corrAns: 'nogo', isiSec: 2.0 },
  { stim: 'octagon.png', shapeName: 'Octagon', corrAns: 'go', isiSec: 1.5 },
];

// Helper to generate 4 multiple-choice options with 1 target and 3 distractors
export function generateRecallOptions(target: FaceItem, facePool: FaceItem[], externalFoils: string[] = []): string[] {
  const distractors = facePool
    .filter(f => f.id !== target.id)
    .map(f => f.name);

  const pool = [...distractors, ...externalFoils];
  // Shuffle and pick 3
  const shuffled = pool.sort(() => Math.random() - 0.5);
  const pickedDistractors = shuffled.slice(0, 3);
  const allOptions = [target.name, ...pickedDistractors];
  return allOptions.sort(() => Math.random() - 0.5);
}
