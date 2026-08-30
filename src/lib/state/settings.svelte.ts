import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '$lib/types/garmin';
import toast from 'svelte-french-toast';

export const settingsState = $state<AppSettings>({
  active_model: 'gemma-4-E2B-it.litertlm',
  medgemma_model: 'medgemma-1.5-4b-it.litertlm',
  huggingface_token: '',
  litert_accelerator: 'Auto',
  litert_max_tokens: 3000,
  ollama_server_url: 'http://localhost:11434',
  tts_voice_style: 'voice_styles/M1.json',
  use_simulated_biometrics: true,
  goblin_name: 'Gribble',
  goblin_personality: 'Tough Love Trainer',
});

export async function loadSettings() {
  try {
    const s = await invoke<AppSettings>('get_settings');
    settingsState.active_model = s.active_model;
    settingsState.medgemma_model = s.medgemma_model;
    settingsState.huggingface_token = s.huggingface_token || '';
    settingsState.litert_accelerator = s.litert_accelerator;
    settingsState.litert_max_tokens = s.litert_max_tokens || 3000;
    settingsState.ollama_server_url = s.ollama_server_url;
    settingsState.tts_voice_style = s.tts_voice_style;
    settingsState.use_simulated_biometrics = s.use_simulated_biometrics;
    settingsState.goblin_name = 'Gribble';
    settingsState.goblin_personality = s.goblin_personality;
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
}

export async function saveSettings() {
  try {
    await invoke('update_settings', {
      settings: {
        active_model: settingsState.active_model,
        medgemma_model: settingsState.medgemma_model,
        huggingface_token: settingsState.huggingface_token?.trim() !== '' ? settingsState.huggingface_token : null,
        litert_accelerator: settingsState.litert_accelerator,
        litert_max_tokens: Number(settingsState.litert_max_tokens) || 3000,
        ollama_server_url: settingsState.ollama_server_url,
        tts_voice_style: settingsState.tts_voice_style,
        use_simulated_biometrics: settingsState.use_simulated_biometrics,
        goblin_name: 'Gribble',
        goblin_personality: settingsState.goblin_personality,
      },
    });
    toast.success('Settings saved!');
  } catch (e) {
    toast.error('Failed to save settings: ' + e);
  }
}
