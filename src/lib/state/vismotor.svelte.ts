// HCP VISMOTOR Reactive State Module (Svelte 5)
import { invoke } from '@tauri-apps/api/core';
import type { VismotorSessionSummary, TodayVismotorSummary } from '$lib/types/vismotor';
import { loadGoblinProfile, loadQuests } from '$lib/state/goblin.svelte';
import toast from 'svelte-french-toast';

export const vismotorState = $state<{
  isLoading: boolean;
  todaySummary: TodayVismotorSummary | null;
  recentSessions: VismotorSessionSummary[];
  activeSessionResult: VismotorSessionSummary | null;
}>({
  isLoading: false,
  todaySummary: null,
  recentSessions: [],
  activeSessionResult: null,
});

export async function loadTodayVismotorSummary() {
  const now = new Date();
  const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  try {
    const summary = await invoke<TodayVismotorSummary>('get_today_vismotor_summary', { date: dateStr });
    vismotorState.todaySummary = summary;
  } catch (err) {
    console.error('Failed to load today VISMOTOR summary:', err);
  }
}

export async function loadRecentVismotorSessions(limit = 20) {
  vismotorState.isLoading = true;
  try {
    const sessions = await invoke<any[]>('get_vismotor_sessions', { date: null, limit });
    vismotorState.recentSessions = sessions.map(s => ({
      id: s.id,
      date: s.date,
      timestamp: s.timestamp,
      timeOfDay: s.timeOfDay || s.time_of_day,
      mode: s.mode,
      totalTrials: s.totalTrials || s.total_trials,
      correctCount: s.correctCount || s.correct_count,
      incorrectCount: s.incorrectCount || s.incorrect_count,
      accuracy: s.accuracy ?? s.accuracy,
      meanRtMs: s.meanRtMs ?? s.mean_rt_ms,
      meanLeftRtMs: s.meanLeftRtMs ?? s.mean_left_rt_ms,
      meanRightRtMs: s.meanRightRtMs ?? s.mean_right_rt_ms,
      hemisphericDifferenceMs: s.hemisphericDifferenceMs ?? s.hemispheric_difference_ms,
      rtStdDevMs: s.rtStdDevMs ?? s.rt_std_dev_ms,
      vismotorScore: s.vismotorScore ?? s.vismotor_score,
      xpAwarded: s.xpAwarded || s.xp_awarded,
      goldAwarded: s.goldAwarded || s.gold_awarded,
      trialDataJson: s.trialDataJson || s.trial_data_json,
    }));
  } catch (err) {
    console.error('Failed to load recent VISMOTOR sessions:', err);
  } finally {
    vismotorState.isLoading = false;
  }
}

export async function recordVismotorSession(summary: VismotorSessionSummary): Promise<VismotorSessionSummary> {
  try {
    await invoke<any>('save_vismotor_session', {
      session: {
        id: null,
        date: summary.date,
        timestamp: summary.timestamp,
        timeOfDay: summary.timeOfDay,
        mode: summary.mode,
        totalTrials: summary.totalTrials,
        correctCount: summary.correctCount,
        incorrectCount: summary.incorrectCount,
        accuracy: summary.accuracy,
        meanRtMs: summary.meanRtMs,
        meanLeftRtMs: summary.meanLeftRtMs,
        meanRightRtMs: summary.meanRightRtMs,
        hemisphericDifferenceMs: summary.hemisphericDifferenceMs,
        rtStdDevMs: summary.rtStdDevMs,
        vismotorScore: summary.vismotorScore,
        xpAwarded: summary.xpAwarded,
        goldAwarded: summary.goldAwarded,
        trialDataJson: summary.trialDataJson || null,
      },
    });

    vismotorState.activeSessionResult = summary;
    await loadTodayVismotorSummary();
    await loadRecentVismotorSessions();
    await loadGoblinProfile();
    await loadQuests();

    toast.success(`⚡ Visuomotor Complete! +${summary.xpAwarded} XP, +${summary.goldAwarded} Gold!`);
    return summary;
  } catch (err: any) {
    console.error('Failed to save VISMOTOR session:', err);
    toast.error('Failed to save visuomotor result: ' + (err?.message || err));
    throw err;
  }
}
