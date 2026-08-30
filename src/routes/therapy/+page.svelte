<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import {
    HeartHandshake, MessageSquare, Brain, Wind, Lock, Sparkles,
    Send, Trash2, Volume2, VolumeX, Shield, Activity, Zap, Moon,
    Clock, Tag, RefreshCw, CheckCircle2, ChevronRight, AlertCircle,
    Flame, Sliders, ArrowRight, Play, Pause, RotateCcw, Calendar, BarChart3,
    ChevronDown, ChevronUp
  } from 'lucide-svelte';
  import {
    therapyState,
    sendTherapyChatMessage,
    clearTherapyChat,
    loadCbtRecords,
    saveCbtRecord,
    deleteCbtRecord,
    requestCbtReframe,
  } from '$lib/state/therapy.svelte';
  import { biometricsState, loadLatestBiometrics, loadBiometricsHistory } from '$lib/state/biometrics.svelte';
  import { goblinState, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { COGNITIVE_DISTORTIONS, SOMATIC_EXERCISES } from '$lib/data/therapy';
  import { speakGuidance, playZenChime, stopSpeech } from '$lib/services/tts';
  import { settingsState } from '$lib/state/settings.svelte';
  import toast from 'svelte-french-toast';
  import type { SomaticExercise } from '$lib/types/therapy';

  // --- Reactive Local State ---
  let activeTab = $state<'chat' | 'cbt' | 'somatic' | 'vault'>('chat');
  let chatScrollContainer: HTMLDivElement | null = $state(null);
  let chatInput = $state('');
  let voiceTtsEnabled = $state(true);
  let isGlanceExpanded = $state(false);

  // Biometrics
  let bio = $derived(biometricsState.current);
  let goblin = $derived(goblinState.profile);

  // Quick Chat Prompts
  const quickPrompts = [
    "Feeling overwhelmed",
    "Untangle an anxious thought",
    "Body feels tense",
    "Mental check-in",
    "Why is my stress high?",
  ];

  // --- CBT Reframer Form State ---
  let cbtSituation = $state('');
  let cbtThought = $state('');
  let cbtSelectedDistortions = $state<string[]>([]);
  let cbtDistressBefore = $state(7);
  let cbtDistressAfter = $state(3);
  let cbtReframedThought = $state('');
  let cbtGoblinAdvice = $state('');
  let cbtSomaticTip = $state('');
  let isReframing = $state(false);

  // --- Somatic Interactive State ---
  let selectedSomaticIndex = $state(0);
  let isSomaticRunning = $state(false);
  let isSomaticPaused = $state(false);
  let somaticStepIndex = $state(0);
  let somaticStepTimeLeft = $state(0);
  let somaticTimerInterval: ReturnType<typeof setInterval> | null = null;

  let currentSomatic = $derived(SOMATIC_EXERCISES[selectedSomaticIndex]);
  let currentSomaticStep = $derived(currentSomatic.steps[somaticStepIndex] || currentSomatic.steps[0]);

  let historyBio = $derived(biometricsState.history);

  // Multi-day Historical Glance Calculations
  let historicalGlance = $derived.by(() => {
    if (!historyBio || historyBio.length === 0) {
      return {
        hasData: false,
        daysCount: 1,
        dateRange: bio?.date ? bio.date : 'No data',
        avgStress: bio?.stress_level ?? 25,
        peakStress: bio?.stress_level ?? 25,
        avgSleepH: Number(((bio?.sleep_duration_sec ?? 25200) / 3600).toFixed(1)),
        avgSleepScore: bio?.sleep_score ?? 80,
        avgHrv: bio?.hrv_rmssd ?? 42,
        dailyRecords: bio ? [bio] : [],
        lastSyncDate: bio?.date || 'unknown',
      };
    }

    const n = historyBio.length;
    const sumStress = historyBio.reduce((acc, h) => acc + h.stress_level, 0);
    const avgStress = Math.round(sumStress / n);
    const peakStress = Math.max(...historyBio.map(h => h.stress_level));

    const sumSleepSec = historyBio.reduce((acc, h) => acc + h.sleep_duration_sec, 0);
    const avgSleepH = Number((sumSleepSec / (n * 3600)).toFixed(1));

    const sumSleepScore = historyBio.reduce((acc, h) => acc + h.sleep_score, 0);
    const avgSleepScore = Math.round(sumSleepScore / n);

    const validHrv = historyBio.map(h => h.hrv_rmssd).filter(h => h > 0);
    const avgHrv = validHrv.length > 0 ? Math.round(validHrv.reduce((a, b) => a + b, 0) / validHrv.length) : (bio?.hrv_rmssd ?? 42);

    const firstDate = historyBio[0]?.date;
    const lastDate = historyBio[historyBio.length - 1]?.date;
    const dateRange = firstDate === lastDate ? firstDate : `${firstDate} → ${lastDate}`;

    return {
      hasData: true,
      daysCount: n,
      dateRange,
      avgStress,
      peakStress,
      avgSleepH,
      avgSleepScore,
      avgHrv,
      dailyRecords: historyBio,
      lastSyncDate: lastDate || bio?.date || 'unknown',
    };
  });

  // Autonomic State Interpretation
  let autonomicState = $derived.by(() => {
    const stress = historicalGlance.avgStress;
    const sleepH = historicalGlance.avgSleepH;

    if (stress > 48 || (stress > 38 && sleepH < 6.5)) {
      return {
        title: 'High Cumulative Load',
        desc: 'Elevated stress and low sleep baseline. Down-regulation recommended.',
        color: 'text-rose-400 bg-rose-500/10 border-rose-500/30',
        badge: 'High Stress',
      };
    } else if (stress > 30) {
      return {
        title: 'Moderate Baseline',
        desc: 'Steady baseline load. Regular somatic breaks recommended.',
        color: 'text-amber-400 bg-amber-500/10 border-amber-500/30',
        badge: 'Moderate',
      };
    } else {
      return {
        title: 'Restorative Reserve',
        desc: 'Healthy vagal tone and low baseline stress.',
        color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30',
        badge: 'Restorative',
      };
    }
  });

  onMount(async () => {
    await loadLatestBiometrics();
    await loadBiometricsHistory(7);
    await loadGoblinProfile();
    await loadCbtRecords();

    // Initial greeting if chat empty
    if (therapyState.messages.length === 0) {
      therapyState.messages = [
        {
          role: 'assistant',
          content: `I am ${goblin?.name || 'Gribble'}, your counselor. In-memory and confidential. What's on your mind?`,
          goblinMood: 'Zen',
        },
      ];
    }
    await scrollToBottom();
  });

  onDestroy(() => {
    stopSomaticExercise();
    stopSpeech();
  });

  // --- Chat Handlers ---
  async function scrollToBottom() {
    await tick();
    if (chatScrollContainer) {
      chatScrollContainer.scrollTop = chatScrollContainer.scrollHeight;
    }
  }

  async function handleSendChat(customText?: string) {
    const textToSend = customText || chatInput.trim();
    if (!textToSend || therapyState.isSending) return;

    chatInput = '';
    await scrollToBottom();

    const response = await sendTherapyChatMessage(textToSend);
    await scrollToBottom();

    if (response && voiceTtsEnabled) {
      speakGuidance(response.slice(0, 150), {
        voiceStyle: settingsState.tts_voice_style || 'voice_styles/F1.json',
      });
    }
  }

  // --- CBT Reframer Handlers ---
  function toggleDistortion(name: string) {
    if (cbtSelectedDistortions.includes(name)) {
      cbtSelectedDistortions = cbtSelectedDistortions.filter((d) => d !== name);
    } else {
      cbtSelectedDistortions = [...cbtSelectedDistortions, name];
    }
  }

  async function generateAiReframe() {
    if (!cbtSituation.trim() || !cbtThought.trim()) {
      toast.error('Please describe the situation and your automatic thought first.');
      return;
    }

    isReframing = true;
    try {
      const result = await requestCbtReframe(
        cbtSituation,
        cbtThought,
        cbtSelectedDistortions
      );

      if (result) {
        cbtReframedThought = result.reframedThought;
        cbtGoblinAdvice = result.goblinAdvice;
        cbtSomaticTip = result.suggestedSomaticAction;
        cbtDistressAfter = Math.max(1, cbtDistressBefore - 4);
        toast.success('Cognitive reframe generated!');
      }
    } finally {
      isReframing = false;
    }
  }

  async function saveCbtExercise() {
    if (!cbtSituation.trim() || !cbtThought.trim() || !cbtReframedThought.trim()) {
      toast.error('Please generate or write a reframed thought before saving.');
      return;
    }

    const todayStr = new Date().toISOString().split('T')[0];
    await saveCbtRecord({
      date: todayStr,
      situation: cbtSituation,
      automaticThought: cbtThought,
      distortions: cbtSelectedDistortions,
      distressBefore: cbtDistressBefore,
      reframedThought: cbtReframedThought,
      distressAfter: cbtDistressAfter,
      garminStress: bio?.stress_level,
      garminHr: bio?.current_hr,
      goblinAdvice: cbtGoblinAdvice,
    });

    // Reset wizard
    cbtSituation = '';
    cbtThought = '';
    cbtSelectedDistortions = [];
    cbtReframedThought = '';
    cbtGoblinAdvice = '';
    cbtSomaticTip = '';
    activeTab = 'vault';
  }

  // --- Somatic Exercise Handlers ---
  function selectSomatic(idx: number) {
    stopSomaticExercise();
    selectedSomaticIndex = idx;
    somaticStepIndex = 0;
  }

  function startSomaticExercise() {
    isSomaticRunning = true;
    isSomaticPaused = false;
    somaticStepIndex = 0;
    somaticStepTimeLeft = currentSomatic.steps[0].durationSec;

    playZenChime();
    if (voiceTtsEnabled) {
      speakGuidance(currentSomatic.spokenCue, {
        voiceStyle: settingsState.tts_voice_style || 'voice_styles/F1.json',
      });
    }

    if (somaticTimerInterval) clearInterval(somaticTimerInterval);
    somaticTimerInterval = setInterval(() => {
      if (!isSomaticPaused) {
        if (somaticStepTimeLeft > 0) {
          somaticStepTimeLeft -= 1;
        } else {
          // Advance step
          if (somaticStepIndex < currentSomatic.steps.length - 1) {
            somaticStepIndex += 1;
            somaticStepTimeLeft = currentSomatic.steps[somaticStepIndex].durationSec;
            if (voiceTtsEnabled) {
              speakGuidance(currentSomatic.steps[somaticStepIndex].cue, {
                voiceStyle: settingsState.tts_voice_style || 'voice_styles/F1.json',
              });
            }
          } else {
            // Loop physiological sigh or finish
            if (currentSomatic.id === 'physiological-sigh') {
              somaticStepIndex = 0;
              somaticStepTimeLeft = currentSomatic.steps[0].durationSec;
            } else {
              stopSomaticExercise();
              toast.success('Somatic exercise completed. Notice your body resting.');
            }
          }
        }
      }
    }, 1000);
  }

  function stopSomaticExercise() {
    isSomaticRunning = false;
    isSomaticPaused = false;
    if (somaticTimerInterval) clearInterval(somaticTimerInterval);
    stopSpeech();
  }
</script>

<div class="max-w-5xl mx-auto p-3 sm:p-4 md:p-6 w-full flex flex-col min-h-full space-y-3 sm:space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between gap-3 border-b border-zinc-800/80 pb-3">
    <div class="flex items-center gap-2.5">
      <div class="w-9 h-9 rounded-xl bg-teal-500/20 border border-teal-500/40 flex items-center justify-center text-lg shadow-[0_0_15px_rgba(20,184,166,0.25)] shrink-0">
        🌿
      </div>
      <div>
        <h1 class="text-base sm:text-lg font-black text-zinc-100 tracking-tight">
          Therapy Mode
        </h1>
      </div>
    </div>

    <!-- Ephemeral Privacy Guarantee Tag -->
    <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-zinc-900/90 border border-zinc-800 text-[11px] text-zinc-400">
      <Lock class="w-3 h-3 text-teal-400 shrink-0" />
      <span>In-memory only</span>
    </div>
  </div>

  <!-- Multi-Day Historical Somatic & Recovery Glance Banner -->
  <div class="p-3 sm:p-4 rounded-2xl border {autonomicState.color} backdrop-blur-md flex flex-col gap-2.5 shadow-sm transition-all">
    <!-- Header with Date Window & Mobile Toggle -->
    <div class="flex items-center justify-between gap-2 border-b border-zinc-800/40 pb-2">
      <div class="flex items-center gap-2 flex-wrap min-w-0">
        <BarChart3 class="w-4 h-4 text-teal-400 shrink-0" />
        <span class="text-xs sm:text-sm font-bold text-zinc-100 truncate">
          Recovery Glance
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-zinc-950/60 border border-zinc-700 text-teal-300 shrink-0">
          {historicalGlance.daysCount}d history
        </span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <div class="hidden sm:flex items-center gap-1 text-[11px] text-zinc-400 font-mono">
          <Calendar class="w-3.5 h-3.5 text-zinc-500" />
          <span>{historicalGlance.dateRange}</span>
        </div>

        <button
          onclick={() => (isGlanceExpanded = !isGlanceExpanded)}
          class="md:hidden flex items-center gap-1 px-2 py-1 rounded-lg bg-zinc-950/60 border border-zinc-800 text-[11px] font-semibold text-zinc-300 hover:text-teal-300 transition-all cursor-pointer"
        >
          <span>{isGlanceExpanded ? 'Less' : 'Details'}</span>
          {#if isGlanceExpanded}
            <ChevronUp class="w-3 h-3" />
          {:else}
            <ChevronDown class="w-3 h-3" />
          {/if}
        </button>
      </div>
    </div>

    <!-- Quick Stats Bar -->
    <div class="flex items-center justify-between gap-2 text-xs">
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-[10px] uppercase font-bold px-2 py-0.5 rounded-full bg-zinc-950/60 border border-current shrink-0">
          {autonomicState.badge}
        </span>
        <span class="text-xs text-zinc-300 truncate sm:hidden">
          {autonomicState.title}
        </span>
      </div>

      <div class="flex items-center gap-2 text-[11px] font-mono shrink-0">
        <span class="text-rose-400">Stress: <strong>{historicalGlance.avgStress}</strong></span>
        <span class="text-zinc-600">•</span>
        <span class="text-indigo-400">Sleep: <strong>{historicalGlance.avgSleepH}h</strong></span>
        <span class="text-zinc-600">•</span>
        <span class="text-teal-400">HRV: <strong>{historicalGlance.avgHrv}ms</strong></span>
      </div>
    </div>

    <!-- Detailed Section -->
    <div class="{isGlanceExpanded ? 'flex' : 'hidden md:flex'} flex-col gap-2.5 pt-1 animate-in fade-in">
      <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
        <div class="space-y-0.5 max-w-xl">
          <span class="hidden sm:inline text-xs font-bold text-zinc-100">{autonomicState.title}</span>
          <p class="text-xs text-zinc-400 leading-normal">{autonomicState.desc}</p>
        </div>

        <!-- Metric Pills -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 shrink-0">
          <div class="bg-zinc-950/70 border border-zinc-800/80 rounded-xl p-2 flex flex-col">
            <span class="text-[10px] text-zinc-400">Avg Stress</span>
            <span class="text-xs sm:text-sm font-bold font-mono text-rose-400">
              {historicalGlance.avgStress}/100
            </span>
            <span class="text-[9px] text-zinc-500 font-mono">Peak: {historicalGlance.peakStress}</span>
          </div>

          <div class="bg-zinc-950/70 border border-zinc-800/80 rounded-xl p-2 flex flex-col">
            <span class="text-[10px] text-zinc-400">Avg Sleep</span>
            <span class="text-xs sm:text-sm font-bold font-mono text-indigo-400">
              {historicalGlance.avgSleepH}h
            </span>
            <span class="text-[9px] text-zinc-500 font-mono">Score: {historicalGlance.avgSleepScore}</span>
          </div>

          <div class="bg-zinc-950/70 border border-zinc-800/80 rounded-xl p-2 flex flex-col">
            <span class="text-[10px] text-zinc-400">Avg HRV</span>
            <span class="text-xs sm:text-sm font-bold font-mono text-teal-400">
              {historicalGlance.avgHrv} ms
            </span>
          </div>

          <div class="bg-zinc-950/70 border border-zinc-800/80 rounded-xl p-2 flex flex-col">
            <span class="text-[10px] text-zinc-400">Battery</span>
            <span class="text-xs sm:text-sm font-bold font-mono text-amber-400">
              {bio?.body_battery ?? 80}%
            </span>
          </div>
        </div>
      </div>

      <!-- Daily Trend Strip -->
      {#if historicalGlance.dailyRecords.length > 1}
        <div class="pt-1.5 border-t border-zinc-800/40">
          <div class="flex items-center justify-between text-[10px] text-zinc-400 mb-1">
            <span class="font-mono text-[10px] text-zinc-400">Daily Stress</span>
            <a href="/" class="text-teal-400 hover:text-teal-200 underline text-[10px]">Sync watch →</a>
          </div>

          <div class="flex items-center gap-1.5 overflow-x-auto pb-1 no-scrollbar md:grid md:grid-cols-7">
            {#each historicalGlance.dailyRecords as record}
              {@const stressVal = record.stress_level}
              {@const isHigh = stressVal > 48}
              {@const isModerate = stressVal > 30 && stressVal <= 48}
              <div class="bg-zinc-950/50 border border-zinc-800/60 rounded-lg p-1.5 flex flex-col items-center text-center shrink-0 w-16 sm:w-auto sm:flex-1">
                <span class="text-[10px] text-zinc-400 font-mono">
                  {record.date.slice(5)}
                </span>
                <div class="w-full bg-zinc-800/50 h-1.5 rounded-full overflow-hidden my-1">
                  <div 
                    class="h-full rounded-full transition-all {isHigh ? 'bg-rose-400' : isModerate ? 'bg-amber-400' : 'bg-emerald-400'}"
                    style="width: {Math.min(100, Math.max(10, stressVal))}%"
                  ></div>
                </div>
                <span class="text-[11px] font-mono font-bold {isHigh ? 'text-rose-400' : isModerate ? 'text-amber-400' : 'text-emerald-400'}">
                  {stressVal}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="flex justify-end pt-1">
          <a href="/" class="text-teal-400 hover:text-teal-200 underline text-[10px]">Sync watch →</a>
        </div>
      {/if}
    </div>
  </div>

  <!-- Tab Navigation -->
  <div class="flex items-center gap-1 sm:gap-1.5 border-b border-zinc-800 pb-1 overflow-x-auto no-scrollbar">
    <button
      onclick={() => (activeTab = 'chat')}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs sm:text-sm font-semibold transition-all shrink-0 cursor-pointer {activeTab === 'chat' ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40 shadow-[0_0_12px_rgba(20,184,166,0.2)]' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
    >
      <MessageSquare class="w-3.5 h-3.5 sm:w-4 sm:h-4" />
      <span>Chat</span>
    </button>

    <button
      onclick={() => (activeTab = 'cbt')}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs sm:text-sm font-semibold transition-all shrink-0 cursor-pointer {activeTab === 'cbt' ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40 shadow-[0_0_12px_rgba(20,184,166,0.2)]' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
    >
      <Brain class="w-3.5 h-3.5 sm:w-4 sm:h-4" />
      <span>CBT</span>
    </button>

    <button
      onclick={() => (activeTab = 'somatic')}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs sm:text-sm font-semibold transition-all shrink-0 cursor-pointer {activeTab === 'somatic' ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40 shadow-[0_0_12px_rgba(20,184,166,0.2)]' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
    >
      <Wind class="w-3.5 h-3.5 sm:w-4 sm:h-4" />
      <span>Somatic</span>
    </button>

    <button
      onclick={() => { activeTab = 'vault'; loadCbtRecords(); }}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs sm:text-sm font-semibold transition-all shrink-0 cursor-pointer {activeTab === 'vault' ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40 shadow-[0_0_12px_rgba(20,184,166,0.2)]' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
    >
      <Sparkles class="w-3.5 h-3.5 sm:w-4 sm:h-4" />
      <span>Vault</span>
      {#if therapyState.cbtRecords.length > 0}
        <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-300 font-mono">
          {therapyState.cbtRecords.length}
        </span>
      {/if}
    </button>
  </div>

  <!-- TAB 1: CHAT -->
  {#if activeTab === 'chat'}
    <div class="flex flex-col h-[520px] sm:h-[580px] lg:h-[620px] bg-zinc-950/70 border border-zinc-800/80 rounded-2xl overflow-hidden shadow-lg">
      <!-- Chat Controls Toolbar -->
      <div class="px-3 sm:px-4 py-2 border-b border-zinc-800/80 bg-zinc-900/40 flex items-center justify-between gap-2">
        <div class="flex items-center gap-2 text-xs text-zinc-400 min-w-0 truncate">
          <span class="w-2 h-2 rounded-full bg-teal-400 animate-pulse shrink-0"></span>
          <span class="truncate">{goblin?.name || 'Gribble'} (Therapist)</span>
        </div>

        <div class="flex items-center gap-1.5 sm:gap-2 shrink-0">
          <button
            onclick={() => (voiceTtsEnabled = !voiceTtsEnabled)}
            class="p-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer {voiceTtsEnabled ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40' : 'text-zinc-500 hover:text-zinc-300'}"
            title={voiceTtsEnabled ? 'Voice enabled' : 'Voice muted'}
          >
            {#if voiceTtsEnabled}
              <Volume2 class="w-3.5 h-3.5" />
            {:else}
              <VolumeX class="w-3.5 h-3.5" />
            {/if}
          </button>

          <button
            onclick={clearTherapyChat}
            class="flex items-center gap-1 text-xs px-2.5 py-1 rounded-lg bg-zinc-800/60 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 transition-all cursor-pointer"
          >
            <Trash2 class="w-3 h-3" />
            <span>Clear</span>
          </button>
        </div>
      </div>

      <!-- Chat Scroll Messages -->
      <div
        bind:this={chatScrollContainer}
        class="flex-1 overflow-y-auto p-3 sm:p-4 space-y-3 text-xs sm:text-sm"
      >
        {#each therapyState.messages as msg}
          <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
            <div class="flex items-center gap-1.5 mb-1 px-1">
              <span class="text-[11px] font-bold {msg.role === 'user' ? 'text-teal-400' : 'text-emerald-400'}">
                {msg.role === 'user' ? 'You' : `${goblin?.name || 'Gribble'} (Therapist)`}
              </span>
              {#if msg.role === 'assistant'}
                <span class="text-[10px] px-1.5 py-0.2 rounded bg-zinc-800 text-teal-300">
                  {msg.goblinMood || 'Zen'}
                </span>
              {/if}
            </div>

            <div
              class="max-w-[92%] sm:max-w-[85%] md:max-w-[75%] px-3.5 sm:px-4 py-2.5 sm:py-3 rounded-2xl whitespace-pre-line leading-relaxed {msg.role === 'user' ? 'bg-teal-600/25 border border-teal-500/40 text-zinc-100 rounded-br-none' : 'bg-zinc-900/90 border border-zinc-800 text-zinc-200 rounded-bl-none shadow-md'}"
            >
              {msg.content}
            </div>
          </div>
        {/each}

        {#if therapyState.isSending}
          <div class="flex items-center gap-2 text-xs text-zinc-400 italic px-2">
            <div class="w-2 h-2 rounded-full bg-teal-400 animate-ping"></div>
            <span>Reflecting...</span>
          </div>
        {/if}
      </div>

      <!-- Quick Prompt Suggestions -->
      <div class="px-3 sm:px-4 py-2 border-t border-zinc-800/80 bg-zinc-900/20 flex items-center gap-1.5 sm:gap-2 overflow-x-auto no-scrollbar">
        {#each quickPrompts as prompt}
          <button
            onclick={() => handleSendChat(prompt)}
            disabled={therapyState.isSending}
            class="text-[11px] px-2.5 sm:px-3 py-1 rounded-full bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-teal-500/40 text-zinc-300 hover:text-teal-300 shrink-0 transition-all cursor-pointer disabled:opacity-50"
          >
            {prompt}
          </button>
        {/each}
      </div>

      <!-- Input Area -->
      <div class="p-2.5 sm:p-3 border-t border-zinc-800 bg-zinc-950/90 flex items-center gap-2">
        <input
          type="text"
          placeholder="Type a message (in-memory only)..."
          bind:value={chatInput}
          onkeydown={(e) => e.key === 'Enter' && handleSendChat()}
          disabled={therapyState.isSending}
          class="flex-1 min-w-0 bg-zinc-900/90 border border-zinc-800 rounded-xl px-3 sm:px-4 py-2 text-xs sm:text-sm text-zinc-100 placeholder:text-zinc-500 focus:outline-none focus:border-teal-500"
        />

        <button
          onclick={() => handleSendChat()}
          disabled={therapyState.isSending || !chatInput.trim()}
          class="px-3.5 sm:px-4 py-2 rounded-xl bg-teal-600 hover:bg-teal-500 text-zinc-950 font-bold text-xs sm:text-sm flex items-center gap-1.5 transition-all cursor-pointer disabled:opacity-50 shrink-0"
        >
          <Send class="w-3.5 h-3.5 sm:w-4 sm:h-4" />
          <span class="hidden sm:inline">Send</span>
        </button>
      </div>
    </div>

  <!-- TAB 2: CBT REFRAMER -->
  {:else if activeTab === 'cbt'}
    <div class="flex-1 overflow-y-auto space-y-4 pb-6">
      <!-- Step 1 & 2: Situation & Negative Thought -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div class="space-y-1.5">
          <label for="cbtSituationInput" class="text-xs font-bold text-zinc-300">1. Situation</label>
          <textarea
            id="cbtSituationInput"
            bind:value={cbtSituation}
            placeholder="What triggered this? (e.g. poor sleep, missed goal, work)"
            rows={2}
            class="w-full bg-zinc-950 border border-zinc-800 rounded-xl p-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-teal-500 resize-none"
          ></textarea>
        </div>

        <div class="space-y-1.5">
          <label for="cbtThoughtInput" class="text-xs font-bold text-zinc-300">2. Negative Thought</label>
          <textarea
            id="cbtThoughtInput"
            bind:value={cbtThought}
            placeholder="What is your mind telling you?"
            rows={2}
            class="w-full bg-zinc-950 border border-zinc-800 rounded-xl p-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-teal-500 resize-none"
          ></textarea>
        </div>
      </div>

      <!-- Step 3: Identify Distortions -->
      <div class="space-y-1.5">
        <div class="text-xs font-bold text-zinc-300">
          3. Distortions
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-1.5">
          {#each COGNITIVE_DISTORTIONS as dist}
            <button
              type="button"
              onclick={() => toggleDistortion(dist.name)}
              class="p-2 rounded-xl border text-left flex items-center justify-between gap-1.5 transition-all cursor-pointer min-w-0 {cbtSelectedDistortions.includes(dist.name) ? 'bg-teal-500/20 border-teal-500 text-teal-200' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'}"
            >
              <div class="flex items-center gap-1.5 min-w-0 truncate">
                <span class="text-sm shrink-0">{dist.icon}</span>
                <span class="text-xs font-semibold truncate">{dist.name}</span>
              </div>
              {#if cbtSelectedDistortions.includes(dist.name)}
                <CheckCircle2 class="w-3.5 h-3.5 text-teal-400 shrink-0" />
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Step 4: Rate Distress Before -->
      <div class="p-3 rounded-xl bg-zinc-900/40 border border-zinc-800 flex items-center justify-between gap-4">
        <span class="text-xs font-bold text-zinc-300">Distress (1–10):</span>

        <div class="flex items-center gap-3 w-48 sm:w-56">
          <input
            type="range"
            min="1"
            max="10"
            bind:value={cbtDistressBefore}
            class="flex-1 accent-teal-500 cursor-pointer"
          />
          <span class="font-mono text-sm font-bold text-teal-400 w-8 text-right shrink-0">{cbtDistressBefore}/10</span>
        </div>
      </div>

      <!-- Action: AI Reframe Button -->
      <div class="flex justify-center pt-1">
        <button
          onclick={generateAiReframe}
          disabled={isReframing || !cbtSituation.trim() || !cbtThought.trim()}
          class="w-full sm:w-auto px-6 py-2.5 rounded-xl bg-teal-600 hover:bg-teal-500 text-zinc-950 font-bold text-xs sm:text-sm flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(20,184,166,0.3)] transition-all cursor-pointer disabled:opacity-50"
        >
          <Sparkles class="w-4 h-4" />
          <span>{isReframing ? 'Reframing...' : 'Reframe Thought'}</span>
        </button>
      </div>

      <!-- Reframed Results Section -->
      {#if cbtReframedThought}
        <div class="space-y-3 p-4 rounded-2xl bg-teal-950/20 border border-teal-500/40 animate-in fade-in">
          <div class="space-y-1">
            <span class="text-xs font-bold uppercase tracking-wider text-teal-300">Reframed Thought</span>
            <textarea
              bind:value={cbtReframedThought}
              rows={2}
              class="w-full bg-zinc-950/80 border border-teal-500/30 rounded-xl p-2.5 text-xs md:text-sm text-zinc-100 focus:outline-none focus:border-teal-400"
            ></textarea>
          </div>

          {#if cbtGoblinAdvice}
            <div class="p-3 rounded-xl bg-zinc-900/80 border border-zinc-800 text-xs text-zinc-300">
              <span class="font-bold text-emerald-400 mr-1.5">Advice:</span>
              {cbtGoblinAdvice}
            </div>
          {/if}

          {#if cbtSomaticTip}
            <div class="p-2.5 rounded-xl bg-cyan-950/30 border border-cyan-500/30 text-xs text-cyan-200 flex items-center gap-2">
              <Wind class="w-3.5 h-3.5 shrink-0" />
              <span><strong>Tip:</strong> {cbtSomaticTip}</span>
            </div>
          {/if}

          <!-- Distress After Slider -->
          <div class="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 flex items-center justify-between gap-4">
            <span class="text-xs font-bold text-zinc-300">Distress (After):</span>

            <div class="flex items-center gap-3 w-48 sm:w-56">
              <input
                type="range"
                min="1"
                max="10"
                bind:value={cbtDistressAfter}
                class="flex-1 accent-emerald-500 cursor-pointer"
              />
              <span class="font-mono text-sm font-bold text-emerald-400 w-8 text-right shrink-0">{cbtDistressAfter}/10</span>
            </div>
          </div>

          <div class="flex justify-end pt-1">
            <button
              onclick={saveCbtExercise}
              class="w-full sm:w-auto justify-center px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-zinc-950 font-bold text-xs sm:text-sm flex items-center gap-2 transition-all cursor-pointer"
            >
              <CheckCircle2 class="w-4 h-4" />
              <span>Save Reframe</span>
            </button>
          </div>
        </div>
      {/if}
    </div>

  <!-- TAB 3: SOMATIC -->
  {:else if activeTab === 'somatic'}
    <div class="flex-1 overflow-y-auto space-y-4 pb-6">
      <!-- Exercise Selection Cards -->
      <div class="flex sm:grid sm:grid-cols-2 md:grid-cols-4 gap-2 overflow-x-auto pb-1 no-scrollbar">
        {#each SOMATIC_EXERCISES as exercise, idx}
          <button
            onclick={() => selectSomatic(idx)}
            class="w-52 sm:w-auto shrink-0 p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between {selectedSomaticIndex === idx ? 'bg-teal-500/20 border-teal-500/60 shadow-[0_0_15px_rgba(20,184,166,0.2)]' : 'bg-zinc-900/60 border-zinc-800 hover:border-zinc-700'}"
          >
            <div>
              <span class="text-[10px] font-bold uppercase tracking-wider text-teal-400 block mb-0.5">{exercise.subtitle}</span>
              <h3 class="text-xs sm:text-sm font-bold text-zinc-100">{exercise.title}</h3>
            </div>

            <div class="mt-2 pt-1.5 border-t border-zinc-800/80 flex items-center justify-between text-[10px] font-mono text-zinc-500">
              <span>{exercise.durationSec}s</span>
              <span class="text-teal-400 font-bold">Select</span>
            </div>
          </button>
        {/each}
      </div>

      <!-- Active Somatic Visualizer Player -->
      <div class="p-4 sm:p-6 rounded-3xl bg-zinc-950/80 border border-zinc-800/80 flex flex-col items-center justify-center text-center space-y-4 shadow-xl relative overflow-hidden">
        <div class="space-y-0.5">
          <h2 class="text-base sm:text-lg font-bold text-zinc-100">{currentSomatic.title}</h2>
          <p class="text-xs text-zinc-400 max-w-md">{currentSomatic.instruction}</p>
        </div>

        <!-- Animated Visual Pacer Ring -->
        <div class="relative w-36 h-36 sm:w-44 sm:h-44 flex items-center justify-center">
          <div
            class="absolute inset-0 rounded-full border-2 border-teal-500/30 transition-transform duration-1000 {isSomaticRunning && currentSomaticStep.visualAction === 'inhale' ? 'scale-125 bg-teal-500/20 border-teal-400' : ''} {isSomaticRunning && currentSomaticStep.visualAction === 'exhale' ? 'scale-75 bg-teal-500/5 border-teal-600' : ''} {isSomaticRunning && currentSomaticStep.visualAction === 'squeeze' ? 'scale-110 bg-amber-500/20 border-amber-500' : ''} {isSomaticRunning && currentSomaticStep.visualAction === 'release' ? 'scale-90 bg-emerald-500/20 border-emerald-400' : ''}"
          ></div>

          <div class="z-10 flex flex-col items-center">
            <span class="text-2xl sm:text-3xl font-black font-mono text-zinc-100">
              {somaticStepTimeLeft}s
            </span>
            <span class="text-xs font-bold uppercase tracking-wider text-teal-300 mt-1">
              {currentSomaticStep.name}
            </span>
          </div>
        </div>

        <!-- Cue Text -->
        <div class="px-3 py-1.5 rounded-xl bg-zinc-900/80 border border-zinc-800 text-xs text-zinc-200 max-w-md">
          {currentSomaticStep.cue}
        </div>

        <!-- Controls -->
        <div class="flex items-center gap-2.5">
          <button
            onclick={() => {
              if (isSomaticRunning) {
                stopSomaticExercise();
              } else {
                startSomaticExercise();
              }
            }}
            class="px-5 py-2.5 rounded-2xl bg-teal-600 hover:bg-teal-500 text-zinc-950 font-bold text-xs sm:text-sm flex items-center gap-2 shadow-[0_0_20px_rgba(20,184,166,0.35)] transition-all cursor-pointer"
          >
            {#if isSomaticRunning}
              <Pause class="w-4 h-4" />
              <span>Pause</span>
            {:else}
              <Play class="w-4 h-4" />
              <span>Start</span>
            {/if}
          </button>

          <button
            onclick={stopSomaticExercise}
            class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 transition-all cursor-pointer"
            title="Reset"
          >
            <RotateCcw class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

  <!-- TAB 4: VAULT -->
  {:else if activeTab === 'vault'}
    <div class="flex-1 overflow-y-auto space-y-3 pb-6">
      <div class="flex items-center justify-between p-3 rounded-2xl bg-zinc-900/60 border border-zinc-800/80 gap-2">
        <h2 class="text-xs sm:text-sm font-bold text-zinc-100 flex items-center gap-2">
          <Sparkles class="w-4 h-4 text-teal-400 shrink-0" />
          <span>Saved Reframes</span>
        </h2>

        <button
          onclick={() => loadCbtRecords()}
          class="p-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-all cursor-pointer shrink-0"
          title="Refresh"
        >
          <RefreshCw class="w-3.5 h-3.5 {therapyState.isLoadingRecords ? 'animate-spin' : ''}" />
        </button>
      </div>

      {#if therapyState.cbtRecords.length === 0}
        <div class="p-8 text-center rounded-2xl border border-zinc-800/80 bg-zinc-950/40 space-y-2">
          <div class="w-10 h-10 rounded-xl bg-zinc-900 mx-auto flex items-center justify-center text-xl">
            📜
          </div>
          <h3 class="text-xs font-bold text-zinc-300">No saved reframes yet.</h3>
          <button
            onclick={() => (activeTab = 'cbt')}
            class="px-3 py-1.5 rounded-xl bg-teal-600/20 text-teal-300 border border-teal-500/40 text-xs font-bold hover:bg-teal-500/30 transition-all cursor-pointer"
          >
            Open CBT
          </button>
        </div>
      {:else}
        <div class="space-y-2.5">
          {#each therapyState.cbtRecords as record}
            <div class="p-3 sm:p-3.5 rounded-2xl bg-zinc-900/70 border border-zinc-800 hover:border-zinc-700/80 transition-all space-y-2.5">
              <div class="flex flex-wrap items-center justify-between gap-2">
                <div class="flex items-center gap-1.5 text-xs text-zinc-400">
                  <Clock class="w-3.5 h-3.5 shrink-0" />
                  <span class="font-mono text-[11px]">{record.date}</span>
                  {#if record.garminStress > 0}
                    <span class="px-1.5 py-0.5 rounded bg-zinc-800 text-rose-300 font-mono text-[10px]">
                      Stress: {record.garminStress}
                    </span>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  <div class="flex items-center gap-1 text-xs font-mono px-2 py-0.5 rounded-lg bg-zinc-950 border border-zinc-800">
                    <span class="text-rose-400 font-bold">{record.distressBefore}</span>
                    <ArrowRight class="w-3 h-3 text-zinc-500" />
                    <span class="text-emerald-400 font-bold">{record.distressAfter}</span>
                  </div>

                  <button
                    onclick={() => deleteCbtRecord(record.id)}
                    class="p-1 rounded-lg text-zinc-500 hover:text-rose-400 hover:bg-rose-500/10 transition-all cursor-pointer"
                    title="Delete"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              <!-- Situation & Thoughts -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                <div class="p-2.5 rounded-xl bg-zinc-950/60 border border-zinc-800/80 space-y-0.5">
                  <span class="text-[10px] uppercase font-bold text-zinc-500">Thought:</span>
                  <p class="text-zinc-300 italic font-medium break-words">"{record.automaticThought}"</p>
                </div>

                <div class="p-2.5 rounded-xl bg-teal-950/20 border border-teal-500/30 space-y-0.5">
                  <span class="text-[10px] uppercase font-bold text-teal-400">Reframe:</span>
                  <p class="text-zinc-200 font-medium break-words">"{record.reframedThought}"</p>
                </div>
              </div>

              <!-- Distortions Tag Chips -->
              {#if record.distortions.length > 0}
                <div class="flex items-center gap-1.5 flex-wrap">
                  {#each record.distortions as dist}
                    <span class="text-[10px] px-2 py-0.5 rounded-full bg-zinc-800/80 text-zinc-400 border border-zinc-700/60">
                      {dist}
                    </span>
                  {/each}
                </div>
              {/if}

              {#if record.goblinAdvice}
                <div class="p-2 rounded-xl bg-zinc-950/40 border border-zinc-800/80 text-xs text-zinc-400">
                  <span class="text-emerald-400 font-bold mr-1">Advice:</span>
                  {record.goblinAdvice}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
