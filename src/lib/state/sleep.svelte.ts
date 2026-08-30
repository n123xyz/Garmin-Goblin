import { biometricsState } from './biometrics.svelte';
import type { GarminBiometrics } from '$lib/types/garmin';

export interface SleepStageDetail {
  seconds: number;
  hours: number;
  minutes: number;
  percentage: number;
  status: 'optimal' | 'moderate' | 'low' | 'high';
  benchmark: string;
  description: string;
}

export interface SleepRecord {
  id: number;
  date: string;
  sleepScore: number;
  totalDurationSec: number;
  formattedDuration: string;
  deep: SleepStageDetail;
  rem: SleepStageDetail;
  light: SleepStageDetail;
  awake: SleepStageDetail;
  restingHr: number;
  hrvRmssd: number;
  hrvStatus: string;
  respirationRate: number;
  spo2: number;
  sleepEfficiencyPct: number;
}

export interface SleepAggregates {
  avgScore: number;
  avgDurationSec: number;
  formattedAvgDuration: string;
  avgDeepPct: number;
  avgRemPct: number;
  avgLightPct: number;
  avgAwakePct: number;
  avgRestingHr: number;
  avgHrvRmssd: number;
  totalNights: number;
  consistencyScore: number; // 0 - 100 consistency index
}

export interface SleepAdvisory {
  sleepHygieneScore: number;
  clinicalBriefing: string;
  goblinAdvice: string;
  actionableProtocol: string[];
}

export const sleepState = $state<{
  selectedDate: string | null;
  timeRangeDays: number;
}>({
  selectedDate: null,
  timeRangeDays: 7,
});

export function formatSecondsToHoursMinutes(sec: number): string {
  if (sec <= 0) return '0h 00m';
  const h = Math.floor(sec / 3600);
  const m = Math.round((sec % 3600) / 60);
  return `${h}h ${String(m).padStart(2, '0')}m`;
}

export function parseSleepRecord(bio: GarminBiometrics): SleepRecord {
  const totalSec = bio.sleep_duration_sec > 0 
    ? bio.sleep_duration_sec 
    : (bio.sleep_deep_sec + bio.sleep_rem_sec + bio.sleep_light_sec + bio.sleep_awake_sec);
  
  const deepSec = bio.sleep_deep_sec;
  const remSec = bio.sleep_rem_sec;
  const lightSec = bio.sleep_light_sec;
  const awakeSec = bio.sleep_awake_sec;

  const validTotal = totalSec > 0 ? totalSec : 1;
  const deepPct = totalSec > 0 ? Math.round((deepSec / validTotal) * 100) : 0;
  const remPct = totalSec > 0 ? Math.round((remSec / validTotal) * 100) : 0;
  const lightPct = totalSec > 0 ? Math.round((lightSec / validTotal) * 100) : 0;
  const awakePct = totalSec > 0 ? Math.round((awakeSec / validTotal) * 100) : 0;

  const targetSec = 8 * 3600; // 8 hours target
  const debtMinutes = Math.round((targetSec - totalSec) / 60);

  const asleepSec = deepSec + remSec + lightSec;
  const efficiency = totalSec > 0 ? Math.min(100, Math.round((asleepSec / totalSec) * 100)) : 0;

  return {
    id: bio.id,
    date: bio.date,
    sleepScore: bio.sleep_score,
    totalDurationSec: totalSec,
    formattedDuration: formatSecondsToHoursMinutes(totalSec),
    deep: {
      seconds: deepSec,
      hours: Math.floor(deepSec / 3600),
      minutes: Math.round((deepSec % 3600) / 60),
      percentage: deepPct,
      status: deepPct >= 15 && deepPct <= 25 ? 'optimal' : deepPct < 15 ? 'low' : 'high',
      benchmark: '15-25% of sleep',
      description: 'Slow-wave delta sleep. Releases growth hormone, repairs musculoskeletal tissue, and strengthens immune defense.',
    },
    rem: {
      seconds: remSec,
      hours: Math.floor(remSec / 3600),
      minutes: Math.round((remSec % 3600) / 60),
      percentage: remPct,
      status: remPct >= 20 && remPct <= 25 ? 'optimal' : remPct < 20 ? 'low' : 'high',
      benchmark: '20-25% of sleep',
      description: 'Rapid Eye Movement sleep. Crucial for memory consolidation, neuroplasticity, cognitive synthesis, and emotional regulation.',
    },
    light: {
      seconds: lightSec,
      hours: Math.floor(lightSec / 3600),
      minutes: Math.round((lightSec % 3600) / 60),
      percentage: lightPct,
      status: lightPct >= 45 && lightPct <= 60 ? 'optimal' : 'moderate',
      benchmark: '45-55% of sleep',
      description: 'Core transition baseline sleep. Regulates autonomic tone, metabolism, and cardiovascular deceleration.',
    },
    awake: {
      seconds: awakeSec,
      hours: Math.floor(awakeSec / 3600),
      minutes: Math.round((awakeSec % 3600) / 60),
      percentage: awakePct,
      status: awakePct <= 10 ? 'optimal' : 'high',
      benchmark: '< 10% of sleep',
      description: 'Micro-arousals and wake episodes. High awake percentages indicate sleep fragmentation or environmental disturbances.',
    },
    restingHr: bio.resting_hr,
    hrvRmssd: bio.hrv_rmssd,
    hrvStatus: bio.hrv_status || 'Balanced',
    respirationRate: bio.respiration_rate > 0 ? bio.respiration_rate : 14,
    spo2: bio.spo2 > 0 ? bio.spo2 : 98,
    sleepEfficiencyPct: efficiency,
  };
}

export function getHistoricalSleepRecords(limitDays = 14): SleepRecord[] {
  const history = biometricsState.history;
  const cur = biometricsState.current;

  const records: SleepRecord[] = [];
  const datesSeen = new Set<string>();

  // Add current/today record if exists
  if (cur && cur.date) {
    records.push(parseSleepRecord(cur));
    datesSeen.add(cur.date);
  }

  for (const h of history) {
    if (!datesSeen.has(h.date)) {
      records.push(parseSleepRecord(h));
      datesSeen.add(h.date);
    }
  }

  // Sort descending by date, then take limit
  records.sort((a, b) => b.date.localeCompare(a.date));
  return records.slice(0, limitDays);
}

export function calculateSleepAggregates(records: SleepRecord[]): SleepAggregates {
  if (records.length === 0) {
    return {
      avgScore: 0,
      avgDurationSec: 0,
      formattedAvgDuration: '0h 00m',
      avgDeepPct: 0,
      avgRemPct: 0,
      avgLightPct: 0,
      avgAwakePct: 0,
      avgRestingHr: 0,
      avgHrvRmssd: 0,
      totalNights: 0,
      consistencyScore: 100,
    };
  }

  let totalScore = 0;
  let totalDur = 0;
  let totalDeep = 0;
  let totalRem = 0;
  let totalLight = 0;
  let totalAwake = 0;
  let totalRhr = 0;
  let totalHrv = 0;
  let countWithData = 0;

  for (const r of records) {
    if (r.totalDurationSec > 0 || r.sleepScore > 0) {
      totalScore += r.sleepScore;
      totalDur += r.totalDurationSec;
      totalDeep += r.deep.percentage;
      totalRem += r.rem.percentage;
      totalLight += r.light.percentage;
      totalAwake += r.awake.percentage;
      totalRhr += r.restingHr;
      totalHrv += r.hrvRmssd;
      countWithData++;
    }
  }

  const n = countWithData > 0 ? countWithData : 1;

  // Calculate consistency: deviation of sleep duration across nights
  const avgDur = totalDur / n;
  let varianceSum = 0;
  for (const r of records) {
    if (r.totalDurationSec > 0) {
      varianceSum += Math.abs(r.totalDurationSec - avgDur);
    }
  }
  const avgDeviationMin = (varianceSum / n) / 60;
  const consistencyScore = Math.max(40, Math.min(100, Math.round(100 - (avgDeviationMin * 0.4))));

  return {
    avgScore: Math.round(totalScore / n),
    avgDurationSec: Math.round(avgDur),
    formattedAvgDuration: formatSecondsToHoursMinutes(Math.round(avgDur)),
    avgDeepPct: Math.round(totalDeep / n),
    avgRemPct: Math.round(totalRem / n),
    avgLightPct: Math.round(totalLight / n),
    avgAwakePct: Math.round(totalAwake / n),
    avgRestingHr: Math.round(totalRhr / n),
    avgHrvRmssd: Math.round(totalHrv / n),
    totalNights: records.length,
    consistencyScore,
  };
}

export function generateSleepAdvisory(
  selectedRecord: SleepRecord | null,
  aggregates: SleepAggregates,
  _curBio: GarminBiometrics | null
): SleepAdvisory {
  // Hygiene score calculation (0-100) based on recorded metrics
  let hygiene = 75;
  if (aggregates.avgScore >= 80) hygiene += 15;
  else if (aggregates.avgScore < 65) hygiene -= 15;
  if (aggregates.consistencyScore >= 80) hygiene += 10;
  if (aggregates.avgDeepPct >= 18) hygiene += 5;
  hygiene = Math.min(100, Math.max(30, hygiene));

  const score = selectedRecord?.sleepScore || aggregates.avgScore;
  const deepPct = selectedRecord?.deep.percentage || aggregates.avgDeepPct;
  const remPct = selectedRecord?.rem.percentage || aggregates.avgRemPct;

  let clinicalBriefing = '';
  let goblinAdvice = '';
  const actionableProtocol: string[] = [];

  if (score >= 80) {
    clinicalBriefing = `Autonomic tone indicates robust parasympathetic recovery. Deep slow-wave sleep (${deepPct}%) and REM duration (${remPct}%) are within high-performance clinical thresholds. Resting heart rate stability reflects minimal systemic inflammation.`;
    goblinAdvice = `GRAH! Look at that glorious sleep potion power! Your brain cells regenerated like a goblin chieftain after a victory feast! You have surplus stamina to conquer heavy dungeon lifting today!`;
    actionableProtocol.push('Maintain consistent circadian anchor by waking at the same time.');
    actionableProtocol.push('Prime daytime cortisol with 10-15 minutes of direct sunlight exposure.');
    actionableProtocol.push('Peak training window is 4-8 hours post-awakening.');
  } else if (score >= 65) {
    clinicalBriefing = `Moderate sleep architecture efficiency observed. Minor slow-wave or REM latency detected. Cardiovascular deceleration was achieved, though sympathetic arousal was intermittently elevated during sleep cycles.`;
    goblinAdvice = `Not terrible, mortal! But your goblin companion notices you woke up a couple of times. Drink plenty of water and don't guzzle coffee past 2:00 PM or I will steal your pillow!`;
    actionableProtocol.push('Implement a hard caffeine cutoff 9 hours prior to sleep.');
    actionableProtocol.push('Keep ambient bedroom temperature cool (65°F-68°F / 18°C-20°C).');
    actionableProtocol.push('Consider 10 minutes of Yoga Nidra pre-sleep breathwork.');
  } else {
    clinicalBriefing = `Significant architectural fragmentation identified. Depressed deep sleep (${deepPct}%) impairs myofibrillar protein synthesis and immune function. Elevated overnight resting heart rate reflects autonomic strain.`;
    goblinAdvice = `BY THE GOBLIN CAVERN! You are shambling like a tired skeleton! Put down the glowing screens, drink herbal tea, and wind down early before your Body Battery drains to ZERO!`;
    actionableProtocol.push('Optimize sleep environment with total darkness, silence, and cool temperatures.');
    actionableProtocol.push('Zero blue-light exposure / mobile screens within 60 minutes of sleep.');
    actionableProtocol.push('Perform 15-minute Guided Sleep Sanctuary Body Scan meditation before sleep.');
  }

  return {
    sleepHygieneScore: hygiene,
    clinicalBriefing,
    goblinAdvice,
    actionableProtocol,
  };
}
