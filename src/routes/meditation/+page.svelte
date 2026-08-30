<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import toast from 'svelte-french-toast';
  import {
    Play, Pause, SkipForward, SkipBack, RotateCcw, Volume2, VolumeX,
    Sparkles, Activity, CheckCircle2, ShieldCheck, Clock,
    Wind, Layers, Info, Compass, ChevronRight, Moon
  } from 'lucide-svelte';
  import { MEDITATION_EXERCISES, type MeditationExercise, type MeditationStep } from '$lib/data/meditation';
  import BoxBreathingVisualizer from '$lib/components/meditation/BoxBreathingVisualizer.svelte';
  import GroundingVisualizer from '$lib/components/meditation/GroundingVisualizer.svelte';
  import LeavesStreamVisualizer from '$lib/components/meditation/LeavesStreamVisualizer.svelte';
  import BodyScanVisualizer from '$lib/components/meditation/BodyScanVisualizer.svelte';
  import { goblinState, setGoblinSpeech, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { settingsState } from '$lib/state/settings.svelte';
  import { speakGuidance, playZenChime, stopSpeech } from '$lib/services/tts';

  // --- Session State ---
  let selectedExerciseIndex = $state(0);
  let currentStepIndex = $state(0);
  let isRunning = $state(false);
  let isPaused = $state(false);
  let timeRemainingInStep = $state(15);
  let totalElapsed = $state(0);
  let voiceEnabled = $state(true);
  let selectedVoiceStyle = $state(settingsState.tts_voice_style || 'voice_styles/F1.json');

  // Box Breathing specific cycle state (4s Inhale, 4s Hold, 4s Exhale, 4s Hold)
  let boxPhase = $state<'Inhale' | 'Hold Full' | 'Exhale' | 'Hold Empty'>('Inhale');
  let boxPhaseSeconds = $state(4);

  let timerInterval: ReturnType<typeof setInterval> | null = null;
  let boxInterval: ReturnType<typeof setInterval> | null = null;

  let currentExercise = $derived(MEDITATION_EXERCISES[selectedExerciseIndex]);
  let currentStep = $derived(currentExercise.steps[currentStepIndex] || currentExercise.steps[0]);

  let totalExerciseDuration = $derived(
    currentExercise.steps.reduce((acc, s) => acc + s.durationSec, 0)
  );

  let stepProgress = $derived(
    currentStep.durationSec > 0
      ? Math.round(((currentStep.durationSec - timeRemainingInStep) / currentStep.durationSec) * 100)
      : 0
  );

  onMount(async () => {
    timeRemainingInStep = currentExercise.steps[0].durationSec;
  });

  onDestroy(() => {
    stopSession();
    stopSpeech();
  });

  // --- Session Flow ---
  function selectExercise(idx: number) {
    stopSession();
    selectedExerciseIndex = idx;
    currentStepIndex = 0;
    timeRemainingInStep = MEDITATION_EXERCISES[idx].steps[0].durationSec;
  }

  function startSession() {
    isRunning = true;
    isPaused = false;
    timeRemainingInStep = currentStep.durationSec;

    playZenChime();

    if (voiceEnabled) {
      speakGuidance(currentStep.spokenGuidance, { voiceStyle: selectedVoiceStyle });
    }

    if (currentExercise.id === 'box-breathing') {
      startBoxCycle();
    }

    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => {
      if (!isPaused) {
        totalElapsed += 1;
        if (timeRemainingInStep > 0) {
          timeRemainingInStep -= 1;
        } else {
          // Advance to next step
          if (currentStepIndex < currentExercise.steps.length - 1) {
            nextStep();
          } else {
            // For box breathing, loop cycles until user finishes or 4 min reached
            if (currentExercise.id === 'box-breathing' && totalElapsed < currentExercise.defaultDurationMin * 60) {
              currentStepIndex = 1; // Loop back to Inhale
              timeRemainingInStep = currentExercise.steps[1].durationSec;
            } else {
              finishSession();
            }
          }
        }
      }
    }, 1000);
  }

  function startBoxCycle() {
    if (boxInterval) clearInterval(boxInterval);
    let boxSeconds = 0;
    boxInterval = setInterval(() => {
      if (isRunning && !isPaused && currentExercise.id === 'box-breathing') {
        boxSeconds = (boxSeconds + 1) % 16;
        if (boxSeconds < 4) {
          boxPhase = 'Inhale';
          boxPhaseSeconds = 4 - (boxSeconds % 4);
        } else if (boxSeconds < 8) {
          boxPhase = 'Hold Full';
          boxPhaseSeconds = 4 - (boxSeconds % 4);
        } else if (boxSeconds < 12) {
          boxPhase = 'Exhale';
          boxPhaseSeconds = 4 - (boxSeconds % 4);
        } else {
          boxPhase = 'Hold Empty';
          boxPhaseSeconds = 4 - (boxSeconds % 4);
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
    if (voiceEnabled && timeRemainingInStep > 5) {
      speakGuidance(`Resuming ${currentStep.title}. Breathe gently.`, { voiceStyle: selectedVoiceStyle });
    }
  }

  function togglePlayPause() {
    if (!isRunning) {
      startSession();
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
    if (boxInterval) {
      clearInterval(boxInterval);
      boxInterval = null;
    }
    stopSpeech();
  }

  function nextStep() {
    if (currentStepIndex < currentExercise.steps.length - 1) {
      currentStepIndex += 1;
      timeRemainingInStep = currentExercise.steps[currentStepIndex].durationSec;
      playZenChime();
      if (voiceEnabled) {
        speakGuidance(currentExercise.steps[currentStepIndex].spokenGuidance, { voiceStyle: selectedVoiceStyle });
      }
    } else {
      finishSession();
    }
  }

  function prevStep() {
    if (currentStepIndex > 0) {
      currentStepIndex -= 1;
      timeRemainingInStep = currentExercise.steps[currentStepIndex].durationSec;
      playZenChime();
      if (voiceEnabled) {
        speakGuidance(currentExercise.steps[currentStepIndex].spokenGuidance, { voiceStyle: selectedVoiceStyle });
      }
    }
  }

  async function finishSession() {
    stopSession();
    playZenChime();

    const xpEarned = Math.max(40, Math.round((totalElapsed / 60) * 20));
    const goldEarned = Math.max(10, Math.round((totalElapsed / 60) * 6));

    setGoblinSpeech(`Ahhh... deep tranquil focus! You earned +${xpEarned} XP & +${goldEarned} Gold, mortal!`);

    if (voiceEnabled) {
      speakGuidance(
        `Session complete. Take this calm, centered presence with you into the rest of your day.`,
        { voiceStyle: selectedVoiceStyle }
      );
    }

    try {
      await loadGoblinProfile();
      toast.success(`Meditation completed! +${xpEarned} XP & +${goldEarned} Gold!`, { duration: 4000 });
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
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800 pb-4">
    <div>
      <h1 class="text-2xl md:text-3xl font-black text-zinc-100 flex items-center gap-2.5">
        <span class="text-2xl md:text-3xl">🧘‍♂️</span>
        SuperTonic Guided Meditation Sanctuary
      </h1>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        4 evidence-based meditation practices with tactile visualizers, SuperTonic TTS voice pacing, and mindful breath tracking.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-xs px-3 py-1.5 rounded-xl font-bold bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 flex items-center gap-1.5">
        <ShieldCheck class="w-4 h-4 text-emerald-400" />
        Offline TTS
      </span>
    </div>
  </div>

  <!-- Garmin Watch Prompt Banner -->
  <div class="bg-gradient-to-r from-cyan-950/40 via-zinc-900 to-purple-950/30 border border-cyan-500/30 rounded-2xl px-4 py-3 flex items-center gap-3 shadow-md">
    <div class="w-8 h-8 rounded-xl bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 flex items-center justify-center shrink-0 text-base">
      ⌚
    </div>
    <div class="text-xs text-zinc-300 leading-snug">
      <strong class="text-cyan-400 font-bold">Garmin Watch Tip:</strong> Start a <strong class="text-zinc-100">Breathwork</strong> or <strong class="text-zinc-100">Meditation</strong> activity on your Garmin watch to record heart rate & HRV.
    </div>
  </div>

  <!-- Main Guided Meditation Stage -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- Left / Center: Interactive Visualizer & Controller -->
    <div class="lg:col-span-8 space-y-4">
      <div class="bg-gradient-to-b from-zinc-900/95 to-zinc-950 border border-zinc-800 rounded-3xl p-6 shadow-2xl relative overflow-hidden flex flex-col justify-between min-h-[480px]">
        <!-- Top Gradient Glow -->
        <div class="absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-cyan-500 via-emerald-500 to-purple-500 opacity-80"></div>

        <!-- Stage Header -->
        <div class="flex items-start justify-between gap-4 z-10">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-[11px] font-mono uppercase px-2.5 py-0.5 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/40">
                Exercise {selectedExerciseIndex + 1} of 4
              </span>
              <span class="text-[11px] font-mono px-2.5 py-0.5 rounded-full bg-zinc-800 text-zinc-400 border border-zinc-700">
                Step {currentStepIndex + 1}/{currentExercise.steps.length}
              </span>
            </div>
            <h2 class="text-2xl md:text-3xl font-black text-zinc-100 mt-1.5 tracking-tight">
              {currentExercise.title}
            </h2>
            <p class="text-xs md:text-sm font-serif italic text-emerald-400 font-medium">
              {currentExercise.subtitle}
            </p>
          </div>

          <!-- TTS Voice Toggle Button -->
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

        <!-- Center: Dynamic Meditation Visualizer -->
        <div class="py-4 flex flex-col items-center justify-center my-auto z-10 w-full">
          {#if currentExercise.id === 'box-breathing'}
            <BoxBreathingVisualizer
              isActive={isRunning && !isPaused}
              phase={boxPhase}
              secondsRemainingInPhase={boxPhaseSeconds}
              phaseProgress={((4 - boxPhaseSeconds) / 4) * 100}
            />
          {:else if currentExercise.id === 'grounding-54321'}
            <GroundingVisualizer
              currentStepIndex={currentStepIndex}
              activeCount={currentStep.count || 5}
            />
          {:else if currentExercise.id === 'leaves-on-a-stream'}
            <LeavesStreamVisualizer />
          {:else if currentExercise.id === 'body-scan'}
            <BodyScanVisualizer
              targetZone={currentStep.targetZone || 'full-body'}
              zoneTitle={currentStep.title}
            />
          {/if}

          <!-- Current Step Instructions Banner -->
          <div class="mt-4 text-center max-w-xl px-4">
            <h4 class="text-xs md:text-sm font-bold text-zinc-200">{currentStep.title}</h4>
            <p class="text-xs text-zinc-400 mt-1 leading-relaxed bg-zinc-900/80 px-4 py-2 rounded-2xl border border-zinc-800/80">
              {currentStep.instruction}
            </p>
          </div>
        </div>

        <!-- Stage Bottom: Timer & Controller -->
        <div class="space-y-3 z-10 pt-2 border-t border-zinc-800/80">
          <div class="space-y-1">
            <div class="flex justify-between text-[11px] font-mono text-zinc-400">
              <span>Step Progress</span>
              <span class="text-emerald-400 font-bold">{formatTime(timeRemainingInStep)} / {formatTime(currentStep.durationSec)}</span>
            </div>
            <div class="w-full h-2 rounded-full bg-zinc-900 overflow-hidden border border-zinc-800">
              <div
                class="h-full bg-gradient-to-r from-cyan-500 via-emerald-400 to-purple-400 transition-all duration-300 rounded-full"
                style="width: {stepProgress}%;"
              ></div>
            </div>
          </div>

          <div class="flex items-center justify-between gap-2 pt-1 flex-wrap">
            <div class="flex items-center gap-1.5">
              <button
                onclick={prevStep}
                disabled={currentStepIndex === 0}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 transition-all cursor-pointer"
                title="Previous Step"
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
                  <span>Begin Meditation</span>
                {/if}
              </button>

              <button
                onclick={nextStep}
                disabled={currentStepIndex === currentExercise.steps.length - 1}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 transition-all cursor-pointer"
                title="Next Step"
              >
                <SkipForward class="w-4 h-4" />
              </button>

              <button
                onclick={stopSession}
                class="p-2.5 rounded-2xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 transition-all cursor-pointer"
                title="Reset Practice"
              >
                <RotateCcw class="w-4 h-4" />
              </button>
            </div>

            <div class="text-xs font-mono text-zinc-400">
              Session Time: <strong class="text-emerald-300">{formatTime(totalElapsed)}</strong>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Sidebar: Exercise Science & Best For -->
    <div class="lg:col-span-4 space-y-4">
      <!-- 1. Best For & Clinical Applications -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center gap-2 border-b border-zinc-800/80 pb-2.5">
          <Compass class="w-4 h-4 text-emerald-400" />
          <h3 class="text-sm font-bold text-zinc-100">Best Applications</h3>
        </div>

        <p class="text-xs text-emerald-300 font-semibold leading-relaxed">
          {currentExercise.bestFor}
        </p>

        <p class="text-xs text-zinc-400 leading-relaxed pt-1">
          {currentExercise.description}
        </p>
      </div>

      <!-- 2. Physiological Mechanism -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center gap-2 border-b border-zinc-800/80 pb-2.5">
          <Info class="w-4 h-4 text-cyan-400" />
          <h3 class="text-sm font-bold text-zinc-100">Why It Works</h3>
        </div>

        <p class="text-xs text-zinc-300 leading-relaxed">
          {currentExercise.whyItWorks}
        </p>
      </div>

      <!-- 3. Target Mindfulness Practice Details -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl space-y-3">
        <div class="flex items-center justify-between border-b border-zinc-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <Moon class="w-4 h-4 text-purple-400" />
            <h3 class="text-sm font-bold text-zinc-100">Practice Details</h3>
          </div>
          <span class="text-[10px] font-mono text-purple-300 font-bold">Mindfulness</span>
        </div>

        <div class="space-y-2 text-xs text-zinc-400">
          <div class="flex justify-between">
            <span>Target Duration:</span>
            <strong class="text-zinc-200">{currentExercise.defaultDurationMin} minutes</strong>
          </div>
          <div class="flex justify-between">
            <span>Total Steps:</span>
            <strong class="text-emerald-300">{currentExercise.steps.length} stages</strong>
          </div>
          <div class="flex justify-between">
            <span>Focus Ratio:</span>
            <strong class="text-zinc-200">{currentExercise.id === 'box-breathing' ? '4-4-4-4 Box' : 'Paced Awareness'}</strong>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- The 4 Meditation Practices Grid Selector -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Layers class="w-5 h-5 text-emerald-400" />
        <h3 class="text-base font-bold text-zinc-100">The 4 Guided Meditation Practices</h3>
      </div>
      <span class="text-xs text-zinc-400">Select any exercise to practice</span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3.5">
      {#each MEDITATION_EXERCISES as ex, idx}
        <button
          onclick={() => selectExercise(idx)}
          class="p-4 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between group {selectedExerciseIndex === idx ? 'bg-emerald-500/10 border-emerald-500/50 shadow-lg shadow-emerald-500/10' : 'bg-zinc-950/70 border-zinc-800/80 hover:border-zinc-700 hover:bg-zinc-900/90'}"
        >
          <div class="flex items-start justify-between gap-2">
            <span class="text-2xl">{ex.icon}</span>
            <span class="text-[10px] font-mono px-2 py-0.5 rounded-md font-bold {selectedExerciseIndex === idx ? 'bg-emerald-500/20 text-emerald-300' : 'bg-zinc-800 text-zinc-400'}">
              {ex.defaultDurationMin} min
            </span>
          </div>

          <div class="mt-4">
            <h4 class="text-sm font-bold text-zinc-200 group-hover:text-emerald-300 transition-colors">
              {ex.title}
            </h4>
            <p class="text-xs text-zinc-400 mt-1 line-clamp-2">
              {ex.subtitle}
            </p>
          </div>
        </button>
      {/each}
    </div>
  </div>
</div>
