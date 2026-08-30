export interface CbtThoughtRecord {
  id: number;
  date: string;
  timestamp: string;
  situation: string;
  automaticThought: string;
  distortions: string[];
  distressBefore: number;
  reframedThought: string;
  distressAfter: number;
  garminStress: number;
  garminHr: number;
  goblinAdvice?: string;
}

export interface CbtThoughtRecordInput {
  date: string;
  situation: string;
  automaticThought: string;
  distortions: string[];
  distressBefore: number;
  reframedThought: string;
  distressAfter: number;
  garminStress?: number;
  garminHr?: number;
  goblinAdvice?: string;
}

export interface CbtReframeResult {
  reframedThought: string;
  identifiedDistortions: string[];
  goblinAdvice: string;
  suggestedSomaticAction: string;
}

export interface DistortionDef {
  id: string;
  name: string;
  shortDesc: string;
  fullExplanation: string;
  example: string;
  reframeQuestion: string;
  icon: string;
}

export interface SomaticExercise {
  id: string;
  title: string;
  subtitle: string;
  durationSec: number;
  nervousSystemTarget: string;
  instruction: string;
  spokenCue: string;
  steps: {
    name: string;
    durationSec: number;
    visualAction: 'inhale' | 'hold' | 'exhale' | 'rest' | 'squeeze' | 'release';
    cue: string;
  }[];
}
