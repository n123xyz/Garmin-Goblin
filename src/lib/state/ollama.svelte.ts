import { invoke } from '@tauri-apps/api/core';
import { settingsState } from '$lib/state/settings.svelte';

export const ollamaState = $state<{
  isHealthy: boolean;
  hasChecked: boolean;
  models: string[];
}>({
  isHealthy: true,
  hasChecked: false,
  models: [],
});

export async function checkOllamaHealth() {
  const isAndroidTauri =
    typeof window !== 'undefined' &&
    ((window as any).__TAURI_INTERNALS__ || /Android/i.test(navigator.userAgent));

  if (isAndroidTauri) {
    ollamaState.isHealthy = true;
    ollamaState.hasChecked = true;
    return;
  }

  try {
    const healthy = await invoke<boolean>('check_ollama_health');
    ollamaState.isHealthy = healthy;
    ollamaState.hasChecked = true;
    if (healthy) {
      await fetchOllamaModels();
    }
  } catch (e) {
    console.error('Failed to check Ollama health:', e);
    ollamaState.isHealthy = false;
    ollamaState.hasChecked = true;
  }
}

export async function fetchOllamaModels() {
  try {
    const rawModels = await invoke<string[]>('list_ollama_models');
    const models = rawModels.filter((m) => !m.includes('litert'));
    ollamaState.models = models;
    const isAndroidTauri = typeof window !== 'undefined' && /Android/i.test(navigator.userAgent);
    const isLitert = settingsState.active_model.includes('litert');

    if (rawModels.includes(settingsState.active_model) || (isAndroidTauri && isLitert)) {
      // Valid model
    } else if (models.length > 0) {
      if (!isLitert || !isAndroidTauri) {
        settingsState.active_model = models[0];
        import('$lib/state/settings.svelte').then((m) => m.saveSettings());
      }
    } else if (isAndroidTauri) {
      if (!isLitert) {
        settingsState.active_model = 'gemma-4-E2B-it.litertlm';
        import('$lib/state/settings.svelte').then((m) => m.saveSettings());
      }
    }
  } catch (e) {
    console.error('Failed to list Ollama models:', e);
  }
}
