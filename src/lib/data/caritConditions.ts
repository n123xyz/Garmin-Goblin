// CARIT Condition Data & Stimuli Definitions (Human Connectome Project Lifespan fMRI Battery)

export interface CaritTrialDef {
  stim: string; // e.g. 'pentagon.png', 'circle.png'
  shapeName: string; // e.g. 'pentagon', 'circle'
  corrAns: 'go' | 'nogo';
  prepotency: number | null; // 2, 3, 4 Go trials prior to this NoGo, or null
  isiSec: number; // Inter-stimulus interval in seconds
}

export const GO_SHAPES = [
  'pentagon.png',
  'hexagon.png',
  'octagon.png',
  'plaque.png',
  'trapezoid.png',
  'parallelogram.png',
];

export const NOGO_SHAPES = [
  'circle.png',
  'square.png',
];

// Resolves a condition stimulus identifier (e.g. 'prevRewNogo', 'neutralNogo', or explicit 'square.png')
export function resolveStimulus(stimRaw: string): { stim: string; shapeName: string; corrAns: 'go' | 'nogo' } {
  let stim = stimRaw.trim();
  if (stim === 'prevRewNogo') {
    stim = 'circle.png';
  } else if (stim === 'neutralNogo') {
    stim = 'square.png';
  } else if (!stim.endsWith('.png') && !stim.endsWith('.bmp')) {
    stim = `${stim}.png`;
  }

  const shapeName = stim.replace(/\.(png|bmp)$/, '');
  const corrAns: 'go' | 'nogo' = NOGO_SHAPES.includes(stim) ? 'nogo' : 'go';
  return { stim, shapeName, corrAns };
}

// 16-Trial Practice Run (practice0.csv)
export const PRACTICE_CONDITIONS: CaritTrialDef[] = [
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 0.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 0.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 0.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 4, isiSec: 2.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 0.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
];

// Daily Standard Cognitive Readiness Test (~36 trials, ~75 seconds)
// Balanced distribution of 2, 3, and 4 prepotency sequences
export const DAILY_TEST_CONDITIONS: CaritTrialDef[] = [
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 4, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
];

// Full 93-Trial HCP Scan Sequence (scan1.csv)
export const FULL_SCAN_CONDITIONS: CaritTrialDef[] = [
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 4.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 4.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 4.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 4, isiSec: 2.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 2.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 4, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 4, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 4, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 2.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 2, isiSec: 1.0 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.0 },
  { stim: 'hexagon.png', shapeName: 'hexagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'circle.png', shapeName: 'circle', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
  { stim: 'trapezoid.png', shapeName: 'trapezoid', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'plaque.png', shapeName: 'plaque', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 2.5 },
  { stim: 'pentagon.png', shapeName: 'pentagon', corrAns: 'go', prepotency: null, isiSec: 2.5 },
  { stim: 'octagon.png', shapeName: 'octagon', corrAns: 'go', prepotency: null, isiSec: 1.0 },
  { stim: 'parallelogram.png', shapeName: 'parallelogram', corrAns: 'go', prepotency: null, isiSec: 1.5 },
  { stim: 'square.png', shapeName: 'square', corrAns: 'nogo', prepotency: 3, isiSec: 1.5 },
];
