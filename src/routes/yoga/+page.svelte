<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import toast from 'svelte-french-toast';
  import { 
    Play, Pause, SkipForward, SkipBack, RotateCcw, Volume2, VolumeX, 
    Sparkles, Activity, CheckCircle2, ShieldCheck, Clock, 
    Flame, Wind, ArrowRight, Layers, SlidersHorizontal, Info
  } from 'lucide-svelte';
  import { YOGA_POSES, type YogaPose } from '$lib/data/yoga';
  import YogaPoseSvg from '$lib/components/yoga/YogaPoseSvg.svelte';
  import { goblinState, setGoblinSpeech, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { settingsState } from '$lib/state/settings.svelte';
  import { speakGuidance, playZenChime, stopSpeech } from '$lib/services/tts';

  // --- Session State ---
  let isRunning = $state(false);
  let isPaused = $state(false);
  let isSinglePoseMode = $state(false);
  let currentPoseIndex = $state(0);
  let timeRemaining = $state(90);
  let totalElapsed = $state(0);
  let voiceEnabled = $state(true);
  let selectedVoiceStyle = $state(settingsState.tts_voice_style || 'voice_styles/F1.json');
  let breathPhase = $state<'Inhale' | 'Hold' | 'Exhale' | 'Rest'>('Inhale');
  let breathProgress = $state(0); // 0 to 100%

  // Custom durations map (pose.id -> seconds)
  let customDurations = $state<Record<string, number>>({});

  let timerInterval: ReturnType<typeof setInterval> | null = null;
  let breathInterval: ReturnType<typeof setInterval> | null = null;

  let currentPose = $derived(YOGA_POSES[currentPoseIndex]);
  let currentTargetDuration = $derived(customDurations[currentPose.id] || currentPose.defaultDurationSec);
  let poseProgress = $derived(
    currentTargetDuration > 0 ? Math.round(((currentTargetDuration - timeRemaining) / currentTargetDuration) * 100) : 0
  );

  let totalRoutineDuration = $derived(
    YOGA_POSES.reduce((acc, p) => acc + (customDurations[p.id] || p.defaultDurationSec), 0)
  );

  let estCalories = $derived(Math.round((totalElapsed / 60) * 4.2)); // ~4.2 kcal/min for yoga flow

  onMount(async () => {
    // Initialize default durations
    const initial: Record<string, number> = {};
    YOGA_POSES.forEach((p) => {
      initial[p.id] = p.defaultDurationSec;
    });
    customDurations = initial;
    timeRemaining = initial[YOGA_POSES[0].id];

    startBreathMetronome();
  });

  onDestroy(() => {
    stopSession();
    stopSpeech();
    if (breathInterval) clearInterval(breathInterval);
  });

  // --- Breath Metronome ---
  function startBreathMetronome() {
    if (breathInterval) clearInterval(breathInterval);
    let step = 0; // 0 to 12s cycle (4s inhale, 2s hold, 4s exhale, 2s rest)
    breathInterval = setInterval(() => {
      step = (step + 1) % 12;
      if (step < 4) {
        breathPhase = 'Inhale';
        breathProgress = Math.round((step / 4) * 100);
      } else if (step < 6) {
        breathPhase = 'Hold';
        breathProgress = Math.round(((step - 4) / 2) * 100);
      } else if (step < 10) {
        breathPhase = 'Exhale';
        breathProgress = Math.round(((step - 6) / 4) * 100);
      } else {
        breathPhase = 'Rest';
        breathProgress = Math.round(((step - 10) / 2) * 100);
      }
    }, 1000);
  }

  // --- Session Control ---
  async function startSession(singleMode = false) {
    isSinglePoseMode = singleMode;
    isRunning = true;
    isPaused = false;
    timeRemaining = customDurations[currentPose.id] || currentPose.defaultDurationSec;

    playZenChime();

    if (voiceEnabled) {
      speakGuidance(currentPose.spokenGuidance, { voiceStyle: selectedVoiceStyle });
    }

    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => {
      if (!isPaused) {
        totalElapsed += 1;
        if (timeRemaining > 0) {
          timeRemaining -= 1;

          // Halfway spoken cue
          const half = Math.floor(currentTargetDuration / 2);
          if (timeRemaining === half && voiceEnabled && currentPose.halfwayCue) {
            speakGuidance(currentPose.halfwayCue, { voiceStyle: selectedVoiceStyle });
          }

          // Countdown cue in last 3 seconds
          if (timeRemaining === 3 && voiceEnabled) {
            speakGuidance('Transitioning in three, two, one', { voiceStyle: selectedVoiceStyle });
          }
        } else {
          // Pose complete
          if (isSinglePoseMode || currentPoseIndex >= YOGA_POSES.length - 1) {
            finishSession();
          } else {
            nextPose();
          }
        }
      }
    }, 1000);
  }

  function pauseSession() {
    isPaused = true;
    stopSpeech();
  }

  function resumeSession() {
    isPaused = false;
    if (voiceEnabled && timeRemaining > 5) {
      speakGuidance(`Resuming ${currentPose.name}. Breathe deeply.`, { voiceStyle: selectedVoiceStyle });
    }
  }

  function togglePlayPause() {
    if (!isRunning) {
      startSession(isSinglePoseMode);
    } else if (isPaused) {
      resumeSession();
    } else {
      pauseSession();
    }
  }

  function stopSession() {
    isRunning = false;
    isPaused = false;
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    stopSpeech();
  }

  function nextPose() {
    if (currentPoseIndex < YOGA_POSES.length - 1) {
      currentPoseIndex += 1;
      timeRemaining = customDurations[YOGA_POSES[currentPoseIndex].id] || YOGA_POSES[currentPoseIndex].defaultDurationSec;
      playZenChime();
      if (voiceEnabled) {
        speakGuidance(YOGA_POSES[currentPoseIndex].spokenGuidance, { voiceStyle: selectedVoiceStyle });
      }
    } else {
      finishSession();
    }
  }

  function prevPose() {
    if (currentPoseIndex > 0) {
      currentPoseIndex -= 1;
      timeRemaining = customDurations[YOGA_POSES[currentPoseIndex].id] || YOGA_POSES[currentPoseIndex].defaultDurationSec;
      playZenChime();
      if (voiceEnabled) {
        speakGuidance(YOGA_POSES[currentPoseIndex].spokenGuidance, { voiceStyle: selectedVoiceStyle });
      }
    }
  }

  function selectPose(idx: number) {
    currentPoseIndex = idx;
    timeRemaining = customDurations[YOGA_POSES[idx].id] || YOGA_POSES[idx].defaultDurationSec;
    if (isRunning) {
      playZenChime();
      if (voiceEnabled) {
        speakGuidance(YOGA_POSES[idx].spokenGuidance, { voiceStyle: selectedVoiceStyle });
      }
    }
  }

  function adjustDuration(poseId: string, deltaSec: number) {
    const current = customDurations[poseId] || 60;
    const pose = YOGA_POSES.find((p) => p.id === poseId);
    const min = pose?.minDurationSec || 15;
    const max = pose?.maxDurationSec || 600;
    const updated = Math.max(min, Math.min(max, current + deltaSec));
    customDurations[poseId] = updated;

    if (currentPose.id === poseId && !isRunning) {
      timeRemaining = updated;
    }
  }

  async function finishSession() {
    stopSession();
    playZenChime();

    const xpEarned = Math.max(50, Math.round((totalElapsed / 60) * 25));
    const goldEarned = Math.max(15, Math.round((totalElapsed / 60) * 8));

    setGoblinSpeech(`Ahhh! Serene flow! You earned +${xpEarned} XP & +${goldEarned} Gold, mortal!`);

    if (voiceEnabled) {
      speakGuidance(
        `Namaste. You have completed your yoga session. You earned ${xpEarned} experience and ${goldEarned} gold for your Goblin companion.`,
        { voiceStyle: selectedVoiceStyle }
      );
    }

    try {
      await loadGoblinProfile();
      toast.success(`Yoga session completed! +${xpEarned} XP & +${goldEarned} Gold!`, { duration: 4000 });
    } catch (e) {
      toast.success(`Session finished! +${xpEarned} XP & +${goldEarned} Gold!`);
    }
  }

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<div class="max-w-5xl mx-auto p-4 md:p-6 w-full space-y-5">
  <!-- Header & Status -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800 pb-4">
    <div>
      <h1 class="text-2xl md:text-3xl font-black text-zinc-100 flex items-center gap-2.5">
        <span class="text-2xl md:text-3xl">🧘‍♀️</span>
        SuperTonic Guided Yoga Studio
      </h1>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        8 foundational Asana practices with neural SuperTonic voice guidance, posture cues, and breathwork pacing.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-xs px-3 py-1.5 rounded-xl font-bold bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 flex items-center gap-1.5">
        <ShieldCheck class="w-4 h-4 text-emerald-400" />
        100% Offline TTS
      </span>
    </div>
  </div>

  <!-- Garmin Watch Prompt Banner -->
  <div class="bg-gradient-to-r from-emerald-950/40 via-zinc-900 to-cyan-950/30 border border-emerald-500/30 rounded-2xl px-4 py-3 flex items-center gap-3 shadow-md">
    <div class="w-8 h-8 rounded-xl bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 flex items-center justify-center shrink-0 text-base">
      ⌚
    </div>
    <div class="text-xs text-zinc-300 leading-snug">
      <strong class="text-emerald-400 font-bold">Garmin Watch Tip:</strong> Start a <strong class="text-zinc-100">Yoga</strong> activity on your Garmin watch to record physiological metrics and sync automatically.
    </div>
  </div>

  <!-- Main Guided Stage -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- Left / Center: Interactive Stage & Refined SVG Architecture -->
    <div class="lg:col-span-8 space-y-4">
      <div class="bg-gradient-to-b from-zinc-900/95 to-zinc-950 border border-zinc-800 rounded-3xl p-6 shadow-2xl relative overflow-hidden flex flex-col justify-between min-h-[460px]">
        <!-- Ambient Top Glow -->
        <div class="absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-emerald-500 via-cyan-500 to-purple-500 opacity-80"></div>

        <!-- Stage Header: Pose Metadata -->
        <div class="flex items-start justify-between gap-4 z-10">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-[11px] font-mono uppercase px-2.5 py-0.5 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/40">
                Pose {currentPose.number} of 8
              </span>
              <span class="text-[11px] font-mono px-2.5 py-0.5 rounded-full bg-zinc-800 text-zinc-400 border border-zinc-700">
                {currentPose.category}
              </span>
            </div>
            <h2 class="text-2xl md:text-3xl font-black text-zinc-100 mt-1.5 tracking-tight">
              {currentPose.name}
            </h2>
            <p class="text-xs md:text-sm font-serif italic text-emerald-400 font-medium">
              {currentPose.sanskritName}
            </p>
          </div>

          <!-- Voice & Pacing Control Hub -->
          <div class="flex items-center gap-2">
            <button
              onclick={() => voiceEnabled = !voiceEnabled}
              class="p-2.5 rounded-2xl border transition-all cursor-pointer {voiceEnabled ? 'bg-purple-500/20 border-purple-500/50 text-purple-300 shadow-md shadow-purple-500/20' : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'}"
              title={voiceEnabled ? 'SuperTonic TTS Active' : 'TTS Muted'}
            >
              {#if voiceEnabled}
                <Volume2 class="w-5 h-5" />
              {:else}
                <VolumeX class="w-5 h-5" />
              {/if}
            </button>
          </div>
        </div>

        <!-- Center: Refined Vector Model Display -->
        <div class="py-4 flex flex-col items-center justify-center my-auto z-10 relative">
          <YogaPoseSvg
            poseId={currentPose.id}
            class="w-52 h-52 md:w-64 md:h-64"
            isActive={isRunning && !isPaused}
            accentColor={currentPose.accentColor}
          />

          <!-- Posture Alignment Pill -->
          <div class="mt-2 text-center max-w-md px-4">
            <p class="text-xs text-zinc-300 font-medium leading-relaxed bg-zinc-900/80 px-3.5 py-1.5 rounded-2xl border border-zinc-800/80">
              <strong class="text-emerald-400 font-semibold">Position:</strong> {currentPose.position}
            </p>
          </div>
        </div>

        <!-- Stage Bottom: Timer & Stage Controller -->
        <div class="space-y-3 z-10 pt-2 border-t border-zinc-800/80">
          <!-- Progress bar for Current Pose -->
          <div class="space-y-1">
            <div class="flex justify-between text-[11px] font-mono text-zinc-400">
              <span>Pose Progress</span>
              <span class="text-emerald-400 font-bold">{formatTime(timeRemaining)} / {formatTime(currentTargetDuration)}</span>
            </div>
            <div class="w-full h-2 rounded-full bg-zinc-900 overflow-hidden border border-zinc-800">
              <div
                class="h-full bg-gradient-to-r from-emerald-500 via-cyan-400 to-emerald-400 transition-all duration-300 rounded-full"
                style="width: {poseProgress}%;"
              ></div>
            </div>
          </div>

          <!-- Controls Button Bar -->
          <div class="flex items-center justify-between gap-2 pt-1 flex-wrap">
            <div class="flex items-center gap-1.5">
              <button
                onclick={prevPose}
                disabled={currentPoseIndex === 0}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 transition-all cursor-pointer"
                title="Previous Pose"
              >
                <SkipBack class="w-4 h-4" />
              </button>

              <button
                onclick={togglePlayPause}
                class="px-5 py-2.5 rounded-2xl font-black text-xs md:text-sm flex items-center gap-2 shadow-lg transition-all cursor-pointer {isRunning && !isPaused ? 'bg-amber-500 hover:bg-amber-400 text-zinc-950 shadow-amber-500/20' : 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-emerald-500/30'}"
              >
                {#if isRunning && !isPaused}
                  <Pause class="w-4 h-4 fill-zinc-950" />
                  <span>Pause</span>
                {:else if isPaused}
                  <Play class="w-4 h-4 fill-zinc-950" />
                  <span>Resume</span>
                {:else}
                  <Play class="w-4 h-4 fill-zinc-950" />
                  <span>Start Guided Flow</span>
                {/if}
              </button>

              <button
                onclick={nextPose}
                disabled={currentPoseIndex === YOGA_POSES.length - 1}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 transition-all cursor-pointer"
                title="Next Pose"
              >
                <SkipForward class="w-4 h-4" />
              </button>

              <button
                onclick={stopSession}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 transition-all cursor-pointer"
                title="Restart Session"
              >
                <RotateCcw class="w-4 h-4" />
              </button>
            </div>

            <!-- Quick Duration Adjustments -->
            <div class="flex items-center gap-1 text-[11px] font-mono">
              <button
                onclick={() => adjustDuration(currentPose.id, -15)}
                class="px-2.5 py-1 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 cursor-pointer"
              >
                -15s
              </button>
              <span class="text-zinc-300 font-bold px-1">{currentTargetDuration}s</span>
              <button
                onclick={() => adjustDuration(currentPose.id, 15)}
                class="px-2.5 py-1 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 cursor-pointer"
              >
                +15s
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Sidebar: Breath Metronome & Pose Description Insights -->
    <div class="lg:col-span-4 space-y-4">
      <!-- 1. Visual Breath Metronome -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center justify-between border-b border-zinc-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <Wind class="w-4 h-4 text-cyan-400" />
            <h3 class="text-sm font-bold text-zinc-100">Pranayama Breath Guide</h3>
          </div>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-bold">
            4-2-4-2 Box
          </span>
        </div>

        <div class="flex items-center gap-4 pt-1">
          <!-- Animated Breathing Circle -->
          <div class="relative w-16 h-16 flex items-center justify-center shrink-0">
            <div 
              class="absolute rounded-full border-2 border-cyan-400/60 transition-all duration-1000 flex items-center justify-center"
              style="width: {breathPhase === 'Inhale' || breathPhase === 'Hold' ? '100%' : '50%'}; height: {breathPhase === 'Inhale' || breathPhase === 'Hold' ? '100%' : '50%'}; background: radial-gradient(circle, rgba(6, 182, 212, 0.2) 0%, transparent 80%);"
            ></div>
            <span class="text-[10px] font-black text-cyan-300 uppercase tracking-wider z-10">{breathPhase}</span>
          </div>

          <div class="space-y-1">
            <div class="text-xs font-bold text-zinc-200">{breathPhase} Phase</div>
            <p class="text-[11px] text-zinc-400 leading-tight">
              Synchronize each asana with calm diaphragmatic nasal breath.
            </p>
          </div>
        </div>
      </div>

      <!-- 2. Pose Anatomical Insights & Benefits -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center gap-2 border-b border-zinc-800/80 pb-2.5">
          <Info class="w-4 h-4 text-emerald-400" />
          <h3 class="text-sm font-bold text-zinc-100">Why This Pose Makes Sense</h3>
        </div>

        <p class="text-xs text-zinc-300 leading-relaxed">
          {currentPose.benefits}
        </p>

        <div class="pt-2 border-t border-zinc-800/80 space-y-1.5">
          <span class="text-[11px] font-bold text-zinc-400 uppercase tracking-wider block">Instructions</span>
          <p class="text-xs text-zinc-400 leading-relaxed">
            {currentPose.description}
          </p>
        </div>
      </div>

      <!-- 3. Telemetry & Goblin Session Stats -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center justify-between border-b border-zinc-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <Activity class="w-4 h-4 text-amber-400" />
            <h3 class="text-sm font-bold text-zinc-100">Session Metrics</h3>
          </div>
          <span class="text-[10px] font-mono text-zinc-400">Total: {formatTime(totalElapsed)}</span>
        </div>

        <div class="grid grid-cols-2 gap-2 text-center">
          <div class="bg-zinc-950/80 p-2.5 rounded-2xl border border-zinc-800/80">
            <span class="text-[10px] uppercase font-bold text-zinc-500 block">Est. Energy</span>
            <span class="text-sm font-black font-mono text-amber-400">{estCalories} kcal</span>
          </div>
          <div class="bg-zinc-950/80 p-2.5 rounded-2xl border border-zinc-800/80">
            <span class="text-[10px] uppercase font-bold text-zinc-500 block">Total Routine</span>
            <span class="text-sm font-black font-mono text-zinc-200">{formatTime(totalRoutineDuration)}</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 8 Foundational Poses Grid Explorer -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Layers class="w-5 h-5 text-emerald-400" />
        <h3 class="text-base font-bold text-zinc-100">The 8 Foundational Yoga Practices</h3>
      </div>
      <span class="text-xs text-zinc-400">Tap any pose to practice individually or jump in sequence</span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3.5">
      {#each YOGA_POSES as pose, idx}
        <button
          onclick={() => selectPose(idx)}
          class="p-4 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between group {currentPoseIndex === idx ? 'bg-emerald-500/10 border-emerald-500/50 shadow-lg shadow-emerald-500/10' : 'bg-zinc-950/70 border-zinc-800/80 hover:border-zinc-700 hover:bg-zinc-900/90'}"
        >
          <div class="flex items-start justify-between gap-2">
            <span class="text-[10px] font-mono px-2 py-0.5 rounded-md font-bold {currentPoseIndex === idx ? 'bg-emerald-500/20 text-emerald-300' : 'bg-zinc-800 text-zinc-400'}">
              #{pose.number}
            </span>
            <span class="text-[10px] font-mono text-zinc-500">
              {customDurations[pose.id] || pose.defaultDurationSec}s
            </span>
          </div>

          <!-- Micro SVG Preview -->
          <div class="py-2 flex justify-center">
            <YogaPoseSvg
              poseId={pose.id}
              class="w-20 h-20 group-hover:scale-105 transition-transform"
              isActive={currentPoseIndex === idx}
              accentColor={pose.accentColor}
            />
          </div>

          <div>
            <h4 class="text-xs font-bold text-zinc-200 group-hover:text-emerald-300 transition-colors">
              {pose.name}
            </h4>
            <p class="text-[10px] font-serif italic text-zinc-400">
              {pose.sanskritName}
            </p>
          </div>
        </button>
      {/each}
    </div>
  </div>
</div>
