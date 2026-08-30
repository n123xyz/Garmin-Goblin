<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { settingsState, loadSettings, saveSettings } from '$lib/state/settings.svelte';
  import { ollamaState, checkOllamaHealth } from '$lib/state/ollama.svelte';
  import { goblinState, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import {
    Settings,
    Cpu,
    ShieldCheck,
    Key,
    RefreshCw,
    Trash2,
    CheckCircle2,
    DownloadCloud,
    ExternalLink,
    Sliders,
    Sparkles,
    Eye,
    EyeOff,
    Mic,
    Server,
    HeartPulse,
    Bot,
    Volume2,
    Play,
    Bell,
    BellRing,
    Calendar as CalendarIcon
  } from 'lucide-svelte';
  import toast from 'svelte-french-toast';
  import { speakGuidance, stopSpeech } from '$lib/services/tts';
  import { 
    notificationState, 
    requestNotificationPermission, 
    saveNotificationSettings, 
    sendTestNotification, 
    loadNotificationSettings 
  } from '$lib/services/notifications.svelte';
  import {
    checkCalendarPermission,
    requestCalendarPermission,
    syncDeviceCalendar,
    calendarState
  } from '$lib/state/calendar.svelte';

  let isSaving = $state(false);
  let showToken = $state(false);
  let isTestingVoice = $state(false);

  // Model States
  let gemmaExists = $state(false);
  let medgemmaExists = $state(false);
  let isDownloadingGemma = $state(false);
  let isDownloadingMedgemma = $state(false);
  let downloadProgressGemma = $state(-1);
  let downloadProgressMedgemma = $state(-1);

  // Supertonic TTS States
  let supertonicReady = $state(false);
  let isDownloadingSupertonic = $state(false);
  let supertonicProgress = $state(-1);

  // Tokenizer States
  let tokenizerExists = $state(false);
  let isDownloadingTokenizer = $state(false);

  let isAndroid = $state(false);
  let showAdvancedMobile = $state(false);
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenSupertonic: UnlistenFn | null = null;

  onMount(() => {
    loadNotificationSettings();
    checkCalendarPermission();
    isAndroid = typeof window !== 'undefined' && (!!(window as any).__TAURI_INTERNALS__ || /Android/i.test(navigator.userAgent));
    loadSettings().then(() => {
      checkModelStatus();
      checkSupertonicModel();
      checkTokenizer();
      if (!isAndroid) {
        checkOllamaHealth();
      }
    });
    loadGoblinProfile();

    // Listen for LiteRT download progress
    listen<{ downloaded: number; total: number; state?: string }>('download_progress', (event) => {
      const { downloaded, total, state } = event.payload;
      if (state) {
        if (state === 'SUCCEEDED') {
          isDownloadingGemma = false;
          isDownloadingMedgemma = false;
          downloadProgressGemma = -1;
          downloadProgressMedgemma = -1;
          checkModelStatus();
          toast.success('LiteRT model downloaded successfully to on-device storage!');
        } else if (state === 'FAILED' || state === 'CANCELLED') {
          isDownloadingGemma = false;
          isDownloadingMedgemma = false;
          downloadProgressGemma = -1;
          downloadProgressMedgemma = -1;
          toast.error(`Model download ${state.toLowerCase()}`);
        }
      } else {
        if (total > 0) {
          const percent = Math.round((downloaded / total) * 100);
          if (isDownloadingMedgemma) downloadProgressMedgemma = percent;
          else downloadProgressGemma = percent;
        }
      }
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    // Listen for Supertonic TTS download progress
    listen<{ downloaded: number; total: number; state?: string }>('supertonic_download_progress', (event) => {
      const { downloaded, total, state } = event.payload;
      isDownloadingSupertonic = true;
      if (state) {
        if (state === 'SUCCEEDED') {
          isDownloadingSupertonic = false;
          supertonicProgress = -1;
          checkSupertonicModel();
          toast.success('Supertonic voice models downloaded successfully!');
        } else if (state === 'FAILED' || state === 'CANCELLED') {
          isDownloadingSupertonic = false;
          supertonicProgress = -1;
          toast.error(`Voice models download ${state.toLowerCase()}`);
        }
      } else {
        if (total > 0) {
          supertonicProgress = Math.round((downloaded / total) * 100);
        } else {
          supertonicProgress = -1;
        }
      }
    }).then((unlisten) => {
      unlistenSupertonic = unlisten;
    });
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
    if (unlistenSupertonic) unlistenSupertonic();
  });

  async function checkModelStatus() {
    try {
      const gemma: any = await invoke('plugin:litert|check_model_exists', {
        payload: { modelPath: 'gemma-4-E2B-it.litertlm' }
      });
      gemmaExists = gemma?.exists || false;
      if (gemma?.isDownloading) isDownloadingGemma = true;

      const med: any = await invoke('plugin:litert|check_model_exists', {
        payload: { modelPath: 'medgemma-1.5-4b-it.litertlm' }
      });
      medgemmaExists = med?.exists || false;
      if (med?.isDownloading) isDownloadingMedgemma = true;
    } catch (e) {
      console.error('Failed to check model status:', e);
    }
  }

  async function testGoblinVoice() {
    try {
      isTestingVoice = true;
      toast('Testing Gribble voice...');
      await speakGuidance("Hehehe! Greetings, champion! I am Gribble, your Garmin Goblin! Let's conquer today's training!", {
        voiceStyle: settingsState.tts_voice_style,
        speed: 1.0,
      });
      isTestingVoice = false;
    } catch (e) {
      isTestingVoice = false;
      toast.error('Voice test failed: ' + e);
    }
  }

  async function downloadModel(modelPath: string) {
    if (modelPath.includes('medgemma') && !settingsState.huggingface_token?.trim()) {
      toast.error('Hugging Face Access Token is required for MedGemma! Enter token above.');
      return;
    }

    try {
      await saveSettings();
      if (modelPath.includes('medgemma')) {
        isDownloadingMedgemma = true;
        downloadProgressMedgemma = 0;
      } else {
        isDownloadingGemma = true;
        downloadProgressGemma = 0;
      }

      await invoke('plugin:litert|download_model', {
        payload: {
          modelPath,
          token: settingsState.huggingface_token?.trim() !== '' ? settingsState.huggingface_token : null,
        }
      });
      toast.success(`Download queued in Android WorkManager!`);
    } catch (e) {
      if (modelPath.includes('medgemma')) isDownloadingMedgemma = false;
      else isDownloadingGemma = false;
      toast.error('Download failed: ' + e);
    }
  }

  async function purgeModel(modelPath: string) {
    try {
      await invoke('plugin:litert|purge_model', { payload: { modelPath } });
      toast.success('Model purged from storage');
      await checkModelStatus();
    } catch (e) {
      toast.error('Failed to purge model: ' + e);
    }
  }

  async function checkSupertonicModel() {
    try {
      const result: { exists: boolean; isDownloading: boolean } = await invoke('plugin:supertonic|is_supertonic_ready');
      supertonicReady = result.exists;
      if (result.isDownloading) {
        isDownloadingSupertonic = true;
      }
    } catch (e) {
      console.error('Failed to check Supertonic model:', e);
    }
  }

  async function downloadSupertonicModel() {
    isDownloadingSupertonic = true;
    supertonicProgress = 0;
    try {
      await invoke('plugin:supertonic|download_supertonic_models', {
        payload: { modelPath: '', downloadUrl: '', token: null }
      });
      toast.success('Supertonic voice models download queued!');
    } catch (e) {
      isDownloadingSupertonic = false;
      toast.error('Failed to download Supertonic models: ' + e);
    }
  }

  async function purgeSupertonicModel() {
    try {
      await invoke('plugin:supertonic|purge_supertonic_models');
      toast.success('Supertonic models purged successfully');
      await checkSupertonicModel();
    } catch (e) {
      toast.error('Failed to purge Supertonic models: ' + e);
    }
  }

  async function checkTokenizer() {
    try {
      tokenizerExists = await invoke<boolean>('check_tokenizer_exists');
    } catch (e) {
      console.error('Failed to check tokenizer:', e);
    }
  }

  async function downloadTokenizer() {
    isDownloadingTokenizer = true;
    try {
      await invoke('download_tokenizer');
      toast.success('Gemma tokenizer downloaded!');
      await checkTokenizer();
    } catch (e) {
      toast.error('Failed to download tokenizer: ' + e);
    } finally {
      isDownloadingTokenizer = false;
    }
  }

  async function purgeTokenizer() {
    try {
      await invoke('delete_tokenizer');
      toast.success('Tokenizer purged');
      await checkTokenizer();
    } catch (e) {
      toast.error('Failed to purge tokenizer: ' + e);
    }
  }

  async function handleSave() {
    isSaving = true;
    await saveSettings();
    await loadGoblinProfile();
    isSaving = false;
  }
</script>

<div class="max-w-4xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800 pb-4">
    <div>
      <h1 class="text-2xl md:text-3xl font-black text-zinc-100 flex items-center gap-2.5">
        <Settings class="w-7 h-7 text-emerald-400" />
        Settings & AI Models
      </h1>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        Configure on-device LiteRT models, Hugging Face credentials, token context window, and companion personality.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-xs px-3 py-1.5 rounded-xl font-bold bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 flex items-center gap-1.5">
        <ShieldCheck class="w-4 h-4" />
        On-Device Privacy
      </span>
    </div>
  </div>

  <!-- 1. Hugging Face Authentication Key (Required for MedGemma) -->
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-amber-950/30 border border-amber-500/40 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2.5">
        <div class="w-9 h-9 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/40 flex items-center justify-center">
          <Key class="w-5 h-5" />
        </div>
        <div>
          <h3 class="text-base font-bold text-zinc-100 flex items-center gap-2">
            Hugging Face Access Token
            <span class="text-[10px] px-2 py-0.5 rounded-full font-extrabold bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase tracking-wider">
              Required for MedGemma
            </span>
          </h3>
          <span class="text-xs text-zinc-400">Authenticates downloads for gated MedGemma & Gemma models</span>
        </div>
      </div>

      <a
        href="https://huggingface.co/settings/tokens"
        target="_blank"
        rel="noopener noreferrer"
        class="inline-flex items-center gap-1 text-xs text-amber-400 hover:text-amber-300 font-semibold hover:underline"
      >
        <span>Get HF Token</span>
        <ExternalLink class="w-3.5 h-3.5" />
      </a>
    </div>

    <div class="space-y-2">
      <div class="relative">
        <input
          id="hfTokenInput"
          type={showToken ? 'text' : 'password'}
          placeholder="hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
          bind:value={settingsState.huggingface_token}
          class="w-full bg-zinc-950 border border-zinc-800 focus:border-amber-500 rounded-2xl px-4 py-3 text-xs md:text-sm text-zinc-100 font-mono pr-12 focus:outline-none transition-colors"
        />
        <button
          type="button"
          onclick={() => showToken = !showToken}
          class="absolute right-3.5 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 cursor-pointer p-1"
          aria-label="Toggle token visibility"
        >
          {#if showToken}
            <EyeOff class="w-4 h-4" />
          {:else}
            <Eye class="w-4 h-4" />
          {/if}
        </button>
      </div>

      <p class="text-[11px] text-zinc-400 leading-relaxed">
        <strong class="text-zinc-300">Why is this needed?</strong> Google MedGemma 1.5 4B IT and Gemma 4 models are distributed under the Gemma open model license. Accept the terms on <a href="https://huggingface.co/litert-community/MedGemma-1.5-4B-IT" target="_blank" class="text-amber-400 hover:underline">Hugging Face MedGemma</a> and paste your Read-permission token here to enable direct on-device downloading.
      </p>
    </div>
  </div>

  <!-- 2. Gribble Notifications & Telemetry Alerts Hub (Zero-AI / Low Overhead) -->
  <div class="bg-gradient-to-br from-zinc-900 via-zinc-900 to-emerald-950/20 border border-emerald-500/40 rounded-3xl p-5 md:p-6 shadow-xl space-y-5">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-4">
      <div class="flex items-center gap-2.5">
        <div class="w-10 h-10 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center text-xl shadow-[0_0_15px_rgba(16,185,129,0.25)]">
          <BellRing class="w-5 h-5" />
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h3 class="text-base md:text-lg font-bold text-zinc-100">{goblinState.profile?.name || 'Gribble'} Notifications & Telemetry Alerts</h3>
            <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 uppercase tracking-wider">
              Zero-AI Engine
            </span>
          </div>
          <span class="text-xs text-zinc-400">Deterministic rule-based reminders for steps, body battery, quests, and sleep without model memory overhead.</span>
        </div>
      </div>

      <!-- Master Alert Toggle -->
      <label class="relative inline-flex items-center cursor-pointer shrink-0">
        <input 
          type="checkbox" 
          bind:checked={notificationState.settings.enabled} 
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="sr-only peer" 
        />
        <div class="w-12 h-6 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500 shadow-inner"></div>
      </label>
    </div>

    <!-- OS Notification Permission Status & Action Strip -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center text-sm font-bold">
          🔔
        </div>
        <div>
          <span class="text-xs font-bold text-zinc-200 block">System Notification Status</span>
          <span class="text-[11px] text-zinc-400">
            Current Permission: 
            <strong class="font-mono {notificationState.permission === 'granted' ? 'text-emerald-400' : notificationState.permission === 'denied' ? 'text-rose-400' : 'text-amber-400'}">
              {notificationState.permission.toUpperCase()}
            </strong>
          </span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        {#if notificationState.permission !== 'granted'}
          <button
            type="button"
            onclick={requestNotificationPermission}
            class="px-3.5 py-1.5 rounded-xl bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-300 border border-emerald-500/40 text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 shadow-sm"
          >
            <Bell class="w-3.5 h-3.5" />
            <span>Enable System Permissions</span>
          </button>
        {/if}

        <button
          type="button"
          onclick={sendTestNotification}
          class="px-3.5 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-200 border border-zinc-700 text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5"
        >
          <span>👹 Send Test Alert</span>
        </button>
      </div>
    </div>

    <!-- Granular Alert Category Toggles Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
      <!-- 1. Steps & Movement -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>👟</span> Step Goals
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">50% and 100% milestone celebrations and inactivity nudges.</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.step_milestones}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>

      <!-- 2. Body Battery Alerts -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>⚡</span> Body Battery
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">Low energy recharge warnings (&lt;25%) and peak readiness (&gt;85%).</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.body_battery_alerts}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>

      <!-- 3. Hydration & Fuel -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>💧</span> Hydration & Meal
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">Daytime water checks and meal ingestion reminders.</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.hydration_reminders}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>

      <!-- 4. Sleep Bedtime Window -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>🌙</span> Bedtime Window
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">Evening wind-down reminder 45m before target bedtime.</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.sleep_bedtime_alerts}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>

      <!-- 5. Quest Completion -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>📜</span> Quest Claims
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">Alerts when completed quests are ready to claim XP and Gold.</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.quest_alerts}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>

      <!-- 6. Stress Spikes -->
      <label class="bg-zinc-950/70 p-3.5 rounded-2xl border border-zinc-800/80 hover:border-zinc-700 flex items-start justify-between gap-3 cursor-pointer transition-colors">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
            <span>🧘</span> High Stress Load
          </span>
          <p class="text-[11px] text-zinc-400 leading-snug">Autonomic load spikes (&gt;75) with box breathing suggestion.</p>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.stress_alerts}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="mt-1 accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </label>
    </div>

    <!-- Alert Frequency & Audio Chime Preferences -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t border-zinc-800/80">
      <div class="space-y-1.5">
        <label for="frequencySelect" class="text-xs font-bold text-zinc-300 uppercase tracking-wider block">Minimum Alert Cooldown</label>
        <select
          id="frequencySelect"
          bind:value={notificationState.settings.frequency_minutes}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500"
        >
          <option value={60}>Every 1 Hour (Frequent)</option>
          <option value={120}>Every 2 Hours (Recommended)</option>
          <option value={240}>Every 4 Hours (Minimal)</option>
          <option value={480}>Every 8 Hours (Major Alerts Only)</option>
        </select>
      </div>

      <div class="flex items-center justify-between bg-zinc-950/70 p-3 rounded-xl border border-zinc-800/80 self-end">
        <div>
          <span class="text-xs font-bold text-zinc-200 block">Goblin Melodic Audio Chime</span>
          <span class="text-[10px] text-zinc-400">Play two-tone synthesizer bell on alert</span>
        </div>
        <input
          type="checkbox"
          bind:checked={notificationState.settings.sound_enabled}
          onchange={() => saveNotificationSettings(notificationState.settings)}
          class="accent-emerald-500 rounded cursor-pointer w-4 h-4"
        />
      </div>
    </div>
  </div>

  <!-- Device Calendar Integration (Samsung / Android Calendar) -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-5">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <CalendarIcon class="w-5 h-5 text-indigo-400" />
        <h3 class="text-base font-bold text-zinc-100">Samsung & Android Device Calendar</h3>
      </div>
      <span class="text-xs px-2.5 py-1 rounded-full font-bold {calendarState.calendarPermissionGranted ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40' : 'bg-zinc-800 text-zinc-400'}">
        {calendarState.calendarPermissionGranted ? 'Linked' : 'Not Linked'}
      </span>
    </div>

    <div class="space-y-3">
      <p class="text-xs text-zinc-400 leading-relaxed">
        Connect your Samsung or Google Calendar to empower MedGemma and Gribble with daily schedule context. When you sync your watch at the end of the day, Garmin Goblin cross-correlates biometric stress spikes with meetings, work blocks, and workouts for retrospective clinical insights.
      </p>

      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2">
        <div>
          {#if calendarState.lastCalendarSyncTime}
            <span class="text-[11px] text-zinc-400">Last synchronized: <strong class="text-zinc-200">{calendarState.lastCalendarSyncTime}</strong></span>
          {:else}
            <span class="text-[11px] text-zinc-400">Read permission queries local Android ContentProvider instances</span>
          {/if}
        </div>

        <div class="flex items-center gap-2">
          {#if !calendarState.calendarPermissionGranted}
            <button
              onclick={async () => {
                const granted = await requestCalendarPermission();
                if (granted) {
                  toast.success('Calendar permission granted!');
                  await syncDeviceCalendar();
                } else {
                  toast.error('Calendar permission was not granted');
                }
              }}
              class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md transition cursor-pointer"
            >
              Grant Calendar Access
            </button>
          {:else}
            <button
              disabled={calendarState.isSyncingCalendar}
              onclick={async () => {
                const count = await syncDeviceCalendar();
                toast.success(`Synced ${count} calendar events!`);
              }}
              class="px-4 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-100 font-bold text-xs border border-zinc-700 transition cursor-pointer flex items-center gap-1.5"
            >
              <RefreshCw class="w-3.5 h-3.5 {calendarState.isSyncingCalendar ? 'animate-spin' : ''}" />
              <span>{calendarState.isSyncingCalendar ? 'Syncing...' : 'Sync Calendar Events'}</span>
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- 3. On-Device LiteRT Model Management -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-5">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Cpu class="w-5 h-5 text-emerald-400" />
        <h3 class="text-base font-bold text-zinc-100">On-Device Edge AI Models (LiteRT)</h3>
      </div>
      <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
        NPU Accelerated
      </span>
    </div>

    <!-- 1. MedGemma 1.5 4B IT Model -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="space-y-1">
        <div class="flex items-center gap-2 flex-wrap">
          <div class="flex items-center gap-1.5">
            <HeartPulse class="w-4 h-4 text-cyan-400" />
            <h4 class="text-sm font-bold text-zinc-200">Google MedGemma 1.5 4B IT</h4>
          </div>
          <span class="text-[11px] px-2 py-0.5 rounded-full font-bold {medgemmaExists ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40' : 'bg-zinc-800 text-zinc-400'}">
            {medgemmaExists ? 'Ready' : 'Not Downloaded'}
          </span>
          <a
            href="https://huggingface.co/litert-community/MedGemma-1.5-4B-IT"
            target="_blank"
            rel="noopener noreferrer"
            class="text-[10px] text-cyan-400/80 hover:text-cyan-300 flex items-center gap-0.5 hover:underline font-mono"
          >
            HF Repo <ExternalLink class="w-2.5 h-2.5" />
          </a>
        </div>
        <p class="text-xs text-zinc-400">Clinical AI engine for HRV recovery diagnostics, autonomic readiness, and cardiology telemetry.</p>
        {#if downloadProgressMedgemma >= 0}
          <div class="w-48 bg-zinc-900 h-2 rounded-full overflow-hidden mt-2 border border-zinc-800">
            <div class="h-full bg-cyan-400 rounded-full transition-all duration-300" style="width: {downloadProgressMedgemma}%;"></div>
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-2 shrink-0">
        {#if isDownloadingMedgemma}
          <span class="text-xs text-cyan-400 font-bold animate-pulse">
            Downloading {downloadProgressMedgemma >= 0 ? `${downloadProgressMedgemma}%` : '...'}
          </span>
        {:else if medgemmaExists}
          <button
            onclick={() => purgeModel('medgemma-1.5-4b-it.litertlm')}
            class="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-rose-950/40 text-rose-400 text-xs font-semibold border border-zinc-800 hover:border-rose-500/30 transition-all cursor-pointer flex items-center gap-1.5"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>Purge</span>
          </button>
        {:else}
          <button
            onclick={() => downloadModel('medgemma-1.5-4b-it.litertlm')}
            class="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-zinc-950 text-xs font-black shadow-[0_0_12px_rgba(6,182,212,0.3)] transition-all cursor-pointer"
          >
            <DownloadCloud class="w-4 h-4" />
            <span>Download</span>
          </button>
        {/if}
      </div>
    </div>

    <!-- 2. Gemma 4 E2B IT Model -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="space-y-1">
        <div class="flex items-center gap-2 flex-wrap">
          <div class="flex items-center gap-1.5">
            <Bot class="w-4 h-4 text-emerald-400" />
            <h4 class="text-sm font-bold text-zinc-200">Gemma 4 E2B IT Model</h4>
          </div>
          <span class="text-[11px] px-2 py-0.5 rounded-full font-bold {gemmaExists ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-zinc-800 text-zinc-400'}">
            {gemmaExists ? 'Ready' : 'Not Downloaded'}
          </span>
          <a
            href="https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm"
            target="_blank"
            rel="noopener noreferrer"
            class="text-[10px] text-emerald-400/80 hover:text-emerald-300 flex items-center gap-0.5 hover:underline font-mono"
          >
            HF Repo <ExternalLink class="w-2.5 h-2.5" />
          </a>
        </div>
        <p class="text-xs text-zinc-400">Powers Goblin persona coaching, smart food & nutrition ingestion, and dialog generation.</p>
        {#if downloadProgressGemma >= 0}
          <div class="w-48 bg-zinc-900 h-2 rounded-full overflow-hidden mt-2 border border-zinc-800">
            <div class="h-full bg-emerald-400 rounded-full transition-all duration-300" style="width: {downloadProgressGemma}%;"></div>
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-2 shrink-0">
        {#if isDownloadingGemma}
          <span class="text-xs text-emerald-400 font-bold animate-pulse">
            Downloading {downloadProgressGemma >= 0 ? `${downloadProgressGemma}%` : '...'}
          </span>
        {:else if gemmaExists}
          <button
            onclick={() => purgeModel('gemma-4-E2B-it.litertlm')}
            class="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-rose-950/40 text-rose-400 text-xs font-semibold border border-zinc-800 hover:border-rose-500/30 transition-all cursor-pointer flex items-center gap-1.5"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>Purge</span>
          </button>
        {:else}
          <button
            onclick={() => downloadModel('gemma-4-E2B-it.litertlm')}
            class="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 text-xs font-black shadow-[0_0_12px_rgba(16,185,129,0.3)] transition-all cursor-pointer"
          >
            <DownloadCloud class="w-4 h-4" />
            <span>Download</span>
          </button>
        {/if}
      </div>
    </div>

    <!-- 3. Gemma Tokenizer -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 flex items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-bold text-zinc-200">Gemma Tokenizer</h4>
          <span class="text-[11px] px-2 py-0.5 rounded-full font-bold {tokenizerExists ? 'bg-emerald-500/20 text-emerald-300' : 'bg-zinc-800 text-zinc-400'}">
            {tokenizerExists ? 'Installed' : 'Missing'}
          </span>
        </div>
        <p class="text-xs text-zinc-400 mt-1">Exact context truncation & token budgeting tokenizer.</p>
      </div>

      <div>
        {#if tokenizerExists}
          <button
            onclick={purgeTokenizer}
            class="px-3 py-1.5 rounded-xl bg-zinc-900 text-rose-400 text-xs font-semibold border border-zinc-800 hover:border-rose-500/30 transition-all cursor-pointer"
          >
            Purge
          </button>
        {:else}
          <button
            onclick={downloadTokenizer}
            disabled={isDownloadingTokenizer}
            class="px-3.5 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-bold border border-zinc-700 transition-all cursor-pointer disabled:opacity-50"
          >
            {isDownloadingTokenizer ? 'Downloading...' : 'Install'}
          </button>
        {/if}
      </div>
    </div>

    <!-- 4. Supertonic TTS Voice Models -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 flex items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2">
          <Mic class="w-4 h-4 text-purple-400" />
          <h4 class="text-sm font-bold text-zinc-200">Supertonic Neural TTS Models</h4>
          <span class="text-[11px] px-2 py-0.5 rounded-full font-bold {supertonicReady ? 'bg-purple-500/20 text-purple-300' : 'bg-zinc-800 text-zinc-400'}">
            {supertonicReady ? 'Ready' : 'Not Downloaded'}
          </span>
        </div>
        <p class="text-xs text-zinc-400 mt-1">ONNX runtime voice generator for real-time spoken Goblin responses.</p>
        {#if supertonicProgress >= 0}
          <div class="w-48 bg-zinc-900 h-2 rounded-full overflow-hidden mt-2 border border-zinc-800">
            <div class="h-full bg-purple-400 rounded-full transition-all duration-300" style="width: {supertonicProgress}%;"></div>
          </div>
        {/if}
      </div>

      <div>
        {#if isDownloadingSupertonic}
          <span class="text-xs text-purple-400 font-bold animate-pulse">
            Downloading {supertonicProgress >= 0 ? `${supertonicProgress}%` : '...'}
          </span>
        {:else if supertonicReady}
          <button
            onclick={purgeSupertonicModel}
            class="px-3 py-1.5 rounded-xl bg-zinc-900 text-rose-400 text-xs font-semibold border border-zinc-800 hover:border-rose-500/30 transition-all cursor-pointer"
          >
            Purge
          </button>
        {:else}
          <button
            onclick={downloadSupertonicModel}
            class="px-3.5 py-1.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-zinc-100 text-xs font-bold transition-all cursor-pointer"
          >
            Download
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- 3. Token Context Limit & Hardware Configuration -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-5">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Sliders class="w-5 h-5 text-emerald-400" />
        <h3 class="text-base font-bold text-zinc-100">Token Context Limit & Hardware</h3>
      </div>
      <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-zinc-800 text-zinc-300 font-mono">
        {settingsState.litert_max_tokens} Tokens
      </span>
    </div>

    <!-- Token Context Limit Slider & Number Input -->
    <div class="bg-zinc-950/80 rounded-2xl p-4 border border-zinc-800/80 space-y-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
        <div>
          <label for="tokenContextLimit" class="text-xs font-bold text-zinc-200 uppercase tracking-wider block">
            Token Context Limit
          </label>
          <span class="text-[11px] text-zinc-400">
            Maximum token budget for chat history, biometrics telemetry, and MedGemma prompt context.
          </span>
        </div>
        <div class="flex items-center gap-2">
          <input
            id="tokenContextLimit"
            type="number"
            min="512"
            max="8192"
            step="128"
            bind:value={settingsState.litert_max_tokens}
            class="w-24 bg-zinc-900 border border-zinc-700 rounded-xl px-3 py-1.5 text-xs text-center font-mono font-bold text-emerald-400 focus:outline-none focus:border-emerald-400"
          />
          <span class="text-xs text-zinc-500">tokens</span>
        </div>
      </div>

      <input
        type="range"
        min="512"
        max="8192"
        step="128"
        bind:value={settingsState.litert_max_tokens}
        class="w-full accent-emerald-400 cursor-pointer"
      />

      <!-- Quick Preset Buttons -->
      <div class="flex items-center gap-2 pt-1 flex-wrap">
        <span class="text-[10px] text-zinc-500 uppercase font-bold">Presets:</span>
        {#each [1024, 2048, 3000, 4096, 6000] as preset}
          <button
            type="button"
            onclick={() => settingsState.litert_max_tokens = preset}
            class="text-[11px] px-2.5 py-0.5 rounded-lg border font-mono transition-all cursor-pointer {settingsState.litert_max_tokens === preset ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50 font-bold' : 'bg-zinc-900 text-zinc-400 border-zinc-800 hover:text-zinc-200'}"
          >
            {preset}
          </button>
        {/each}
      </div>
    </div>

    <!-- Hardware Accelerator & Companion Configurations -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Personality Style -->
      <div class="space-y-1.5">
        <label for="goblinPersonality" class="text-xs font-bold text-zinc-400 uppercase tracking-wider">Gribble Personality Mode</label>
        <select
          id="goblinPersonality"
          bind:value={settingsState.goblin_personality}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-emerald-500"
        >
          <option>Tough Love Trainer</option>
          <option>Empathetic Goblin Therapist</option>
          <option>Cavern Counselor (CBT & Somatic)</option>
          <option>Mischievous Dungeon Imp</option>
          <option>Zen Cave Shaman</option>
          <option>High-Energy Hype Goblin</option>
        </select>
      </div>

      <!-- LiteRT Hardware Accelerator -->
      <div class="space-y-1.5">
        <label for="acceleratorSelect" class="text-xs font-bold text-zinc-400 uppercase tracking-wider">LiteRT Hardware Accelerator</label>
        <select
          id="acceleratorSelect"
          bind:value={settingsState.litert_accelerator}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-emerald-500"
        >
          <option>Auto (NPU → GPU → CPU)</option>
          <option>NPU (Neural Processing Unit)</option>
          <option>GPU (OpenCL / Vulkan)</option>
          <option>CPU (Multi-threaded ARM)</option>
        </select>
      </div>

      <!-- TTS Voice Style -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <label for="ttsVoiceSelect" class="text-xs font-bold text-zinc-400 uppercase tracking-wider">TTS Voice Profile (SuperTonic)</label>
          <button
            onclick={testGoblinVoice}
            disabled={isTestingVoice}
            class="text-[11px] px-2 py-0.5 rounded-lg bg-purple-500/20 text-purple-300 border border-purple-500/40 hover:bg-purple-500/30 flex items-center gap-1 font-semibold transition-all cursor-pointer disabled:opacity-50"
          >
            <Volume2 class="w-3 h-3" />
            <span>{isTestingVoice ? 'Playing...' : 'Test Voice'}</span>
          </button>
        </div>
        <select
          id="ttsVoiceSelect"
          bind:value={settingsState.tts_voice_style}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 font-mono"
        >
          <optgroup label="Male Voices (M1 - M5)">
            <option value="voice_styles/M1.json">M1 (Gruff Goblin / Deep)</option>
            <option value="voice_styles/M2.json">M2 (Warrior / Energetic)</option>
            <option value="voice_styles/M3.json">M3 (Balanced Coach)</option>
            <option value="voice_styles/M4.json">M4 (Warm / Steady)</option>
            <option value="voice_styles/M5.json">M5 (Resonant / Calm)</option>
          </optgroup>
          <optgroup label="Female Voices (F1 - F5)">
            <option value="voice_styles/F1.json">F1 (Default / Expressive)</option>
            <option value="voice_styles/F2.json">F2 (Bright / Upbeat)</option>
            <option value="voice_styles/F3.json">F3 (Warm / Mentor)</option>
            <option value="voice_styles/F4.json">F4 (Zen / Gentle)</option>
            <option value="voice_styles/F5.json">F5 (Crisp / Athletic)</option>
          </optgroup>
        </select>
      </div>
    </div>

    <!-- Save Button -->
    <div class="pt-3 border-t border-zinc-800 flex justify-end">
      <button
        onclick={handleSave}
        disabled={isSaving}
        class="px-6 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-black text-xs md:text-sm shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-all cursor-pointer disabled:opacity-50"
      >
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>

  <!-- 4. Advanced Mode Toggle (Mobile) -->
  {#if isAndroid}
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl flex items-center justify-between">
      <div>
        <h4 class="text-sm md:text-base font-bold text-zinc-100">Advanced Mode</h4>
        <p class="text-xs text-zinc-400 mt-0.5">Enable off-device Ollama server testing and custom server URLs.</p>
      </div>
      <label class="relative inline-flex items-center cursor-pointer">
        <input type="checkbox" bind:checked={showAdvancedMobile} class="sr-only peer" />
        <div class="w-11 h-6 bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
      </label>
    </div>
  {/if}

  <!-- 5. Desktop / Ollama Fallback (Shown on Desktop or when Advanced Mode is enabled on Mobile) -->
  {#if !isAndroid || showAdvancedMobile}
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <div class="flex items-center gap-2">
          <Server class="w-5 h-5 text-zinc-400" />
          <h3 class="text-base font-bold text-zinc-100">Ollama Fallback Server</h3>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full {ollamaState.isHealthy ? 'bg-emerald-400 shadow-[0_0_8px_#34d399]' : 'bg-rose-400'}"></span>
          <span class="text-xs font-mono {ollamaState.isHealthy ? 'text-emerald-400' : 'text-rose-400'}">
            {ollamaState.isHealthy ? 'Connected' : 'Offline'}
          </span>
        </div>
      </div>

      <div class="space-y-3">
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={settingsState.ollama_server_url}
            placeholder="http://10.0.2.2:11434 (Emulator) or LAN IP"
            class="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-emerald-500"
          />
          <button
            type="button"
            onclick={checkOllamaHealth}
            class="px-4 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-xs font-bold text-zinc-200 border border-zinc-700 cursor-pointer flex items-center gap-1.5"
          >
            <RefreshCw class="w-3.5 h-3.5" />
            <span>Check</span>
          </button>
        </div>
        <p class="text-[11px] text-zinc-500">
          When running on Linux or remote environments, Garmin Goblin connects to your Ollama server.
        </p>
      </div>
    </div>
  {/if}
</div>
