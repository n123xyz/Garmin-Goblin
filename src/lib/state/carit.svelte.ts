// CARIT Reactive State Module (Svelte 5)
import { invoke } from '@tauri-apps/api/core';
import type { CaritSessionSummary, TodayCaritSummary } from '$lib/types/carit';
import { loadGoblinProfile, loadQuests } from '$lib/state/goblin.svelte';
import toast from 'svelte-french-toast';

export const caritState = $state<{
  isLoading: boolean;
  todaySummary: TodayCaritSummary | null;
  recentSessions: CaritSessionSummary[];
  activeSessionResult: CaritSessionSummary | null;
}>({
  isLoading: false,
  todaySummary: null,
  recentSessions: [],
  activeSessionResult: null,
});

export async function loadTodayCaritSummary() {
  const now = new Date();
  const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  try {
    const summary = await invoke<TodayCaritSummary>('get_today_carit_summary', { date: dateStr });
    caritState.todaySummary = summary;
  } catch (err) {
    console.error('Failed to load today CARIT summary:', err);
  }
}

export async function loadRecentCaritSessions(limit = 20) {
  caritState.isLoading = true;
  try {
    const sessions = await invoke<any[]>('get_carit_sessions', { date: null, limit });
    caritState.recentSessions = sessions.map(s => ({
      id: s.id,
      date: s.date,
      timestamp: s.timestamp,
      timeOfDay: s.timeOfDay || s.time_of_day,
      mode: s.mode,
      totalTrials: s.totalTrials || s.total_trials,
      hitCount: s.hitCount || s.hit_count,
      missCount: s.missCount || s.miss_count,
      falseAlarmCount: s.falseAlarmCount || s.false_alarm_count,
      corrRejectCount: s.corrRejectCount || s.corr_reject_count,
      totalAccuracy: s.totalAccuracy ?? s.total_accuracy,
      goAccuracy: s.goAccuracy ?? s.go_accuracy,
      nogoAccuracy: s.nogoAccuracy ?? s.nogo_accuracy,
      meanRtMs: s.meanRtMs ?? s.mean_rt_ms,
      medianRtMs: s.medianRtMs ?? s.median_rt_ms,
      rtStdDevMs: s.rtStdDevMs ?? s.rt_std_dev_ms,
      dPrime: s.dPrime ?? s.d_prime,
      cognitiveScore: s.cognitiveScore ?? s.cognitive_score,
      xpAwarded: s.xpAwarded || s.xp_awarded,
      goldAwarded: s.goldAwarded || s.gold_awarded,
      trialDataJson: s.trialDataJson || s.trial_data_json,
    }));
  } catch (err) {
    console.error('Failed to load recent CARIT sessions:', err);
  } finally {
    caritState.isLoading = false;
  }
}

export async function recordCaritSession(summary: CaritSessionSummary): Promise<CaritSessionSummary> {
  try {
    const saved = await invoke<any>('save_carit_session', {
      session: {
        id: null,
        date: summary.date,
        timestamp: summary.timestamp,
        timeOfDay: summary.timeOfDay,
        mode: summary.mode,
        totalTrials: summary.totalTrials,
        hitCount: summary.hitCount,
        missCount: summary.missCount,
        falseAlarmCount: summary.falseAlarmCount,
        corrRejectCount: summary.corrRejectCount,
        totalAccuracy: summary.totalAccuracy,
        goAccuracy: summary.goAccuracy,
        nogoAccuracy: summary.nogoAccuracy,
        meanRtMs: summary.meanRtMs,
        medianRtMs: summary.medianRtMs,
        rtStdDevMs: summary.rtStdDevMs,
        dPrime: summary.dPrime,
        cognitiveScore: summary.cognitiveScore,
        xpAwarded: summary.xpAwarded,
        goldAwarded: summary.goldAwarded,
        trialDataJson: summary.trialDataJson || null,
      },
    });

    caritState.activeSessionResult = summary;
    await loadTodayCaritSummary();
    await loadRecentCaritSessions();
    await loadGoblinProfile();
    await loadQuests();

    toast.success(`🧠 CARIT Complete! +${summary.xpAwarded} XP, +${summary.goldAwarded} Gold!`);
    return summary;
  } catch (err: any) {
    console.error('Failed to save CARIT session:', err);
    toast.error('Failed to save test result: ' + (err?.message || err));
    throw err;
  }
}
