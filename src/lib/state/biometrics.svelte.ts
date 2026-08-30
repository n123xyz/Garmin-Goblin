import { invoke } from '@tauri-apps/api/core';
import type { GarminActivity, GarminBiometrics, HealthInsight } from '$lib/types/garmin';
import { loadGoblinProfile, loadQuests, setGoblinSpeech } from './goblin.svelte';
import toast from 'svelte-french-toast';

export const biometricsState = $state<{
  current: GarminBiometrics | null;
  history: GarminBiometrics[];
  activities: GarminActivity[];
  insights: HealthInsight[];
  isAnalyzing: boolean;
}>({
  current: null,
  history: [],
  activities: [],
  insights: [],
  isAnalyzing: false,
});

export async function loadLatestBiometrics() {
  try {
    const bio = await invoke<GarminBiometrics>('get_latest_biometrics');
    biometricsState.current = bio;
  } catch (e) {
    console.error('Failed to load latest biometrics:', e);
  }
}

export async function loadBiometricsHistory(limit = 30) {
  try {
    const history = await invoke<GarminBiometrics[]>('get_biometrics_history', { limit });
    biometricsState.history = history || [];
  } catch (e) {
    console.error('Failed to load biometrics history:', e);
  }
}

export async function loadActivities(limit = 20) {
  try {
    const acts = await invoke<GarminActivity[]>('get_activities', { limit });
    biometricsState.activities = acts || [];
  } catch (e) {
    console.error('Failed to load activities:', e);
  }
}

export async function loadHealthInsights(limit = 10) {
  try {
    const list = await invoke<HealthInsight[]>('get_latest_health_insights', { limit });
    biometricsState.insights = list || [];
  } catch (e) {
    console.error('Failed to load health insights:', e);
  }
}

export async function generateMedGemmaInsight() {
  biometricsState.isAnalyzing = true;
  try {
    toast('MedGemma analyzing Garmin biometrics...');
    const insight = await invoke<HealthInsight>('generate_medgemma_health_insight');
    biometricsState.insights = [insight, ...biometricsState.insights];
    setGoblinSpeech(insight.goblin_reaction);
    toast.success('MedGemma Clinical Briefing Ready!');
  } catch (e) {
    toast.error('MedGemma analysis failed: ' + e);
  } finally {
    biometricsState.isAnalyzing = false;
  }
}
