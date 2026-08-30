// HCP FACENAME Reactive State Module (Svelte 5)
import { invoke } from '@tauri-apps/api/core';
import type { FaceNameSessionSummary, TodayFaceNameSummary } from '$lib/types/facename';
import { loadGoblinProfile, loadQuests } from '$lib/state/goblin.svelte';
import toast from 'svelte-french-toast';

export const facenameState = $state<{
  isLoading: boolean;
  todaySummary: TodayFaceNameSummary | null;
  recentSessions: FaceNameSessionSummary[];
  activeSessionResult: FaceNameSessionSummary | null;
}>({
  isLoading: false,
  todaySummary: null,
  recentSessions: [],
  activeSessionResult: null,
});

export async function loadTodayFaceNameSummary() {
  const now = new Date();
  const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  try {
    const summary = await invoke<TodayFaceNameSummary>('get_today_facename_summary', { date: dateStr });
    facenameState.todaySummary = summary;
  } catch (err) {
    console.error('Failed to load today FACENAME summary:', err);
  }
}

export async function loadRecentFaceNameSessions(limit = 20) {
  facenameState.isLoading = true;
  try {
    const sessions = await invoke<any[]>('get_facename_sessions', { date: null, limit });
    facenameState.recentSessions = sessions.map(s => ({
      id: s.id,
      date: s.date,
      timestamp: s.timestamp,
      timeOfDay: s.timeOfDay || s.time_of_day,
      mode: s.mode,
      totalMemorized: s.totalMemorized || s.total_memorized,
      totalRecalled: s.totalRecalled || s.total_recalled,
      correctRecallCount: s.correctRecallCount || s.correct_recall_count,
      recallAccuracy: s.recallAccuracy ?? s.recall_accuracy,
      meanRecallRtMs: s.meanRecallRtMs ?? s.mean_recall_rt_ms,
      medianRecallRtMs: s.medianRecallRtMs ?? s.median_recall_rt_ms,
      distractorAccuracy: s.distractorAccuracy ?? s.distractor_accuracy,
      memoryScore: s.memoryScore ?? s.memory_score,
      xpAwarded: s.xpAwarded || s.xp_awarded,
      goldAwarded: s.goldAwarded || s.gold_awarded,
      trialDataJson: s.trialDataJson || s.trial_data_json,
    }));
  } catch (err) {
    console.error('Failed to load recent FACENAME sessions:', err);
  } finally {
    facenameState.isLoading = false;
  }
}

export async function recordFaceNameSession(summary: FaceNameSessionSummary): Promise<FaceNameSessionSummary> {
  try {
    await invoke<any>('save_facename_session', {
      session: {
        id: null,
        date: summary.date,
        timestamp: summary.timestamp,
        timeOfDay: summary.timeOfDay,
        mode: summary.mode,
        totalMemorized: summary.totalMemorized,
        totalRecalled: summary.totalRecalled,
        correctRecallCount: summary.correctRecallCount,
        recallAccuracy: summary.recallAccuracy,
        meanRecallRtMs: summary.meanRecallRtMs,
        medianRecallRtMs: summary.medianRecallRtMs,
        distractorAccuracy: summary.distractorAccuracy,
        memoryScore: summary.memoryScore,
        xpAwarded: summary.xpAwarded,
        goldAwarded: summary.goldAwarded,
        trialDataJson: summary.trialDataJson || null,
      },
    });

    facenameState.activeSessionResult = summary;
    await loadTodayFaceNameSummary();
    await loadRecentFaceNameSessions();
    await loadGoblinProfile();
    await loadQuests();

    toast.success(`🧩 Memory Complete! +${summary.xpAwarded} XP, +${summary.goldAwarded} Gold!`);
    return summary;
  } catch (err: any) {
    console.error('Failed to save FACENAME session:', err);
    toast.error('Failed to save memory result: ' + (err?.message || err));
    throw err;
  }
}
