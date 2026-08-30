<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { caritState, loadTodayCaritSummary, loadRecentCaritSessions, recordCaritSession } from '$lib/state/carit.svelte';
  import { biometricsState, loadLatestBiometrics } from '$lib/state/biometrics.svelte';
  import { goblinState, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { CaritEngine, type EngineState } from '$lib/services/caritEngine';
  import { DAILY_TEST_CONDITIONS, PRACTICE_CONDITIONS, FULL_SCAN_CONDITIONS, type CaritTrialDef } from '$lib/data/caritConditions';
  import type { CaritTrialRecord, CaritSessionSummary } from '$lib/types/carit';
  import { 
    Brain, 
    Zap, 
    Target, 
    ShieldAlert, 
    Trophy, 
    History, 
    Play, 
    RotateCcw, 
    Sparkles, 
    CheckCircle2, 
    XCircle, 
    ArrowRight, 
    Activity, 
    Clock, 
    Coins,
    HelpCircle
  } from 'lucide-svelte';

  let engine: CaritEngine | null = null;
  let activeTab = $state<'test' | 'history' | 'rules'>('test');
  let selectedMode = $state<'daily' | 'practice' | 'full_scan'>('daily');

  // Engine state tracking
  let engineState = $state<EngineState>('IDLE');
  let countdownNumber = $state<number>(3);
  let currentTrial = $state<CaritTrialDef | null>(null);
  let currentTrialIndex = $state<number>(0);
  let totalTrialsCount = $state<number>(0);
  let lastTrialRecord = $state<CaritTrialRecord | null>(null);
  let practiceFeedbackText = $state<string | null>(null);
  let feedbackTimer: any = null;

  // Touch & Flash visual feedback
  let isActionActive = $state(false);

  let todaySummary = $derived(caritState.todaySummary);
  let recentSessions = $derived(caritState.recentSessions);
  let latestBio = $derived(biometricsState.current);
  let goblin = $derived(goblinState.profile);
  let lastSummary = $derived(caritState.activeSessionResult);

  onMount(async () => {
    await Promise.all([
      loadTodayCaritSummary(),
      loadRecentCaritSessions(20),
      loadLatestBiometrics(),
      loadGoblinProfile(),
    ]);

    engine = new CaritEngine({
      onStateChange: (state) => {
        engineState = state;
      },
      onCountdownTick: (count) => {
        countdownNumber = count;
      },
      onTrialStart: (index, trial) => {
        currentTrialIndex = index;
        currentTrial = trial;
      },
      onTrialOutcome: (record) => {
        lastTrialRecord = record;
        if (selectedMode === 'practice') {
          showPracticeFeedback(record);
        }
      },
      onFinished: async (summary) => {
        await recordCaritSession(summary);
      },
    });

    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    if (engine) engine.stop();
    if (feedbackTimer) clearTimeout(feedbackTimer);
    window.removeEventListener('keydown', handleKeyDown);
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.code === 'Space' || e.key === ' ') {
      e.preventDefault();
      triggerUserAction();
    }
  }

  function triggerUserAction() {
    if (!engine) return;
    isActionActive = true;
    setTimeout(() => { isActionActive = false; }, 120);
    engine.handleUserAction();
  }

  function showPracticeFeedback(record: CaritTrialRecord) {
    if (feedbackTimer) clearTimeout(feedbackTimer);
    if (record.outcome === 'Hit') {
      practiceFeedbackText = `✅ Hit! (${record.reactionTimeMs}ms)`;
    } else if (record.outcome === 'CorrectRejection') {
      practiceFeedbackText = `🛡️ Correctly Inhibited!`;
    } else if (record.outcome === 'FalseAlarm') {
      practiceFeedbackText = `❌ False Alarm! (Pressed on ${record.shapeName})`;
    } else if (record.outcome === 'Miss') {
      practiceFeedbackText = `⚠️ Missed Go Target!`;
    }

    feedbackTimer = setTimeout(() => {
      practiceFeedbackText = null;
    }, 700);
  }

  function startTest(mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    selectedMode = mode;
    lastTrialRecord = null;
    practiceFeedbackText = null;
    caritState.activeSessionResult = null;

    let conditions = DAILY_TEST_CONDITIONS;
    if (mode === 'practice') conditions = PRACTICE_CONDITIONS;
    else if (mode === 'full_scan') conditions = FULL_SCAN_CONDITIONS;

    totalTrialsCount = conditions.length;
    engine?.start(conditions, mode);
  }

  function stopTest() {
    engine?.stop();
    engineState = 'IDLE';
  }

  function formatTime(isoStr: string): string {
    try {
      const d = new Date(isoStr);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  }
</script>

<div class="max-w-5xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Top Banner / Header -->
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-indigo-950/40 border border-indigo-500/20 rounded-3xl p-5 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-indigo-500/20 border border-indigo-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(99,102,241,0.3)]">
        🧠
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          CARIT Focus Arena
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">
          Daily Cognitive Readiness & Motor Response Inhibition Assessment (HCP Standard).
        </p>
      </div>
    </div>

    <!-- Quick Stat Summary -->
    <div class="flex items-center gap-3 bg-zinc-950/80 p-3 rounded-2xl border border-zinc-800 shrink-0">
      <div class="flex items-center gap-2 text-indigo-400 font-bold text-xs md:text-sm">
        <Brain class="w-4 h-4" />
        {#if todaySummary?.hasCompletedToday}
          <span class="text-emerald-400 font-black font-mono">Score {todaySummary.latestScore}/100</span>
          <span class="text-[10px] text-zinc-400 font-normal">({todaySummary.latestMeanRtMs?.toFixed(0)}ms RT)</span>
        {:else}
          <span class="text-amber-400 font-medium">Ready for Today</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Navigation Tabs -->
  <div class="flex items-center gap-2 border-b border-zinc-800/80 pb-3">
    <button
      onclick={() => activeTab = 'test'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'test' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <Play class="w-4 h-4" />
      <span>Cognitive Test</span>
    </button>
    <button
      onclick={() => activeTab = 'history'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'history' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <History class="w-4 h-4" />
      <span>History & Trends</span>
    </button>
    <button
      onclick={() => activeTab = 'rules'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'rules' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <HelpCircle class="w-4 h-4" />
      <span>Task Rules & Stimuli</span>
    </button>
  </div>

  {#if activeTab === 'test'}
    {#if engineState === 'IDLE' && !lastSummary}
      <!-- Start / Mode Selection Card -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 text-center shadow-xl">
        <div class="max-w-xl mx-auto space-y-3">
          <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/30 text-indigo-400 text-xs font-bold">
            <Zap class="w-3.5 h-3.5" />
            Standardized Cognitive Check-in
          </div>
          <h2 class="text-2xl md:text-3xl font-black text-zinc-100">
            Daily Cognitive Readiness Test
          </h2>
          <p class="text-xs md:text-sm text-zinc-400 leading-relaxed">
            Test your executive motor inhibition, attention alertness, and neural reaction speed. Complete once daily or before training to correlate with your Garmin Body Battery and autonomic recovery.
          </p>
        </div>

        <!-- Visual Rules Summary Banner -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 max-w-2xl mx-auto text-left">
          <div class="p-4 rounded-2xl bg-emerald-950/20 border border-emerald-500/30 flex items-center gap-3.5">
            <div class="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center text-xl shrink-0 font-bold text-emerald-400">
              ⚡
            </div>
            <div>
              <h3 class="text-sm font-bold text-emerald-400">GO SHAPES (Tap Fast!)</h3>
              <p class="text-[11px] text-zinc-300 mt-0.5">Pentagon, Hexagon, Octagon, Plaque, Trapezoid, Parallelogram</p>
            </div>
          </div>

          <div class="p-4 rounded-2xl bg-rose-950/20 border border-rose-500/30 flex items-center gap-3.5">
            <div class="w-10 h-10 rounded-xl bg-rose-500/20 flex items-center justify-center text-xl shrink-0 font-bold text-rose-400">
              🛑
            </div>
            <div>
              <h3 class="text-sm font-bold text-rose-400">NO-GO SHAPES (Do NOT Tap)</h3>
              <p class="text-[11px] text-zinc-300 mt-0.5">Circle and Square (Withhold all presses)</p>
            </div>
          </div>
        </div>

        <!-- Launch Buttons -->
        <div class="pt-4 flex flex-col sm:flex-row items-center justify-center gap-3 max-w-lg mx-auto">
          <button
            onclick={() => startTest('daily')}
            class="w-full sm:w-auto flex-1 py-3.5 px-6 rounded-2xl bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] text-white font-black text-sm md:text-base flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(99,102,241,0.4)] transition-all"
          >
            <Play class="w-5 h-5 fill-current" />
            <span>Start Daily Test (~75s)</span>
          </button>

          <button
            onclick={() => startTest('practice')}
            class="w-full sm:w-auto py-3.5 px-5 rounded-2xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-bold text-xs md:text-sm flex items-center justify-center gap-2 transition-all border border-zinc-700"
          >
            <span>Practice Run (16 trials)</span>
          </button>
        </div>
      </div>
    {:else if engineState !== 'IDLE' && engineState !== 'FINISHED'}
      <!-- Active In-Game Test Viewport -->
      <div class="relative bg-zinc-950 border border-zinc-800 rounded-3xl overflow-hidden shadow-2xl flex flex-col items-center justify-between min-h-[440px] md:min-h-[500px] p-6 select-none">
        
        <!-- Header Progress -->
        <div class="w-full flex items-center justify-between z-10">
          <div class="flex items-center gap-2">
            <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
              Trial {currentTrialIndex + 1} / {totalTrialsCount}
            </span>
            <span class="text-xs font-mono text-zinc-400 uppercase font-semibold">
              {selectedMode.replace('_', ' ')}
            </span>
          </div>

          <button
            onclick={stopTest}
            class="text-xs px-3 py-1 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 transition-colors"
          >
            Cancel
          </button>
        </div>

        <!-- Main Display Arena (Center) -->
        <div class="flex-1 flex flex-col items-center justify-center w-full my-4 relative">
          {#if engineState === 'COUNTDOWN'}
            <div class="flex flex-col items-center justify-center animate-pulse">
              <span class="text-7xl md:text-8xl font-black font-mono text-indigo-400 drop-shadow-[0_0_30px_rgba(99,102,241,0.6)]">
                {countdownNumber}
              </span>
              <span class="text-xs text-zinc-400 font-bold mt-2 tracking-widest uppercase">Get Ready...</span>
            </div>
          {:else if engineState === 'SHAPE' && currentTrial}
            <!-- 600ms Stimulus Shape Display -->
            <div class="relative flex items-center justify-center w-56 h-56 md:w-72 md:h-72">
              <img
                src={`/carit/${currentTrial.stim}`}
                alt={currentTrial.shapeName}
                class="w-full h-full object-contain filter drop-shadow-[0_0_20px_rgba(255,255,255,0.2)]"
              />
            </div>
          {:else if engineState === 'SHOULDER' || engineState === 'ISI'}
            <!-- Fixation Cross (+) -->
            <div class="flex items-center justify-center w-56 h-56 md:w-72 md:h-72">
              <span class="text-6xl md:text-7xl font-mono text-zinc-300 font-bold select-none opacity-80">
                +
              </span>
            </div>
          {/if}

          <!-- Practice Feedback Overlay -->
          {#if selectedMode === 'practice' && practiceFeedbackText}
            <div class="absolute bottom-2 px-4 py-1.5 rounded-full bg-zinc-900/90 border border-indigo-500/40 text-xs font-bold text-zinc-100 backdrop-blur-md shadow-lg animate-bounce">
              {practiceFeedbackText}
            </div>
          {/if}
        </div>

        <!-- Mobile Tactile Tap Surface / Keyboard Helper -->
        <div class="w-full flex flex-col items-center gap-2 z-10">
          <button
            onpointerdown={(e) => { e.preventDefault(); triggerUserAction(); }}
            class="w-full py-5 rounded-2xl font-black text-base md:text-lg flex items-center justify-center gap-2 border transition-all active:scale-[0.98] {isActionActive ? 'bg-emerald-500 text-zinc-950 border-emerald-400 shadow-[0_0_25px_rgba(16,185,129,0.7)]' : 'bg-zinc-900 hover:bg-zinc-800 text-zinc-100 border-zinc-700 shadow-md'}"
          >
            <Zap class="w-5 h-5" />
            <span>TAP / HIT (or press SPACE)</span>
          </button>
          <span class="text-[10px] text-zinc-400">Withhold tapping when you see a Circle or Square</span>
        </div>
      </div>
    {:else if lastSummary}
      <!-- Test Results & Psychometrics Summary -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
        <!-- Result Header -->
        <div class="flex flex-col md:flex-row items-center justify-between gap-4 border-b border-zinc-800 pb-5">
          <div class="flex items-center gap-3.5">
            <div class="w-14 h-14 rounded-2xl bg-indigo-500/20 border border-indigo-500/40 flex items-center justify-center text-3xl shadow-[0_0_20px_rgba(99,102,241,0.3)]">
              🎯
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl md:text-2xl font-black text-zinc-100">
                  Cognitive Readiness Assessment
                </h2>
                <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-indigo-500/20 text-indigo-300">
                  {lastSummary.mode === 'daily' ? 'Daily Check-in' : lastSummary.mode}
                </span>
              </div>
              <p class="text-xs text-zinc-400 mt-0.5">
                Completed at {formatTime(lastSummary.timestamp)} • Evaluated against clinical motor inhibition norms
              </p>
            </div>
          </div>

          <!-- Score Gauge Badge -->
          <div class="flex items-center gap-3 bg-zinc-950 p-3 rounded-2xl border border-zinc-800">
            <div class="text-right">
              <span class="text-[10px] uppercase font-bold text-zinc-400 block">Readiness Index</span>
              <span class="text-2xl font-black font-mono text-emerald-400">{lastSummary.cognitiveScore}/100</span>
            </div>
          </div>
        </div>

        <!-- Core Metric Cards (4 Grid) -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4">
          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Clock class="w-3.5 h-3.5 text-indigo-400" />
              <span>Mean Reaction Time</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {lastSummary.meanRtMs.toFixed(0)} <span class="text-xs font-normal text-zinc-400">ms</span>
            </div>
            <div class="text-[10px] text-zinc-400">
              Median: {lastSummary.medianRtMs}ms • Var: ±{lastSummary.rtStdDevMs}ms
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <ShieldAlert class="w-3.5 h-3.5 text-rose-400" />
              <span>Inhibitory Control</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-emerald-400">
              {(lastSummary.nogoAccuracy * 100).toFixed(1)}%
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.corrRejectCount} Rejections / {lastSummary.falseAlarmCount} False Alarms
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Target class="w-3.5 h-3.5 text-emerald-400" />
              <span>Go Hit Accuracy</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {(lastSummary.goAccuracy * 100).toFixed(1)}%
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.hitCount} Hits / {lastSummary.missCount} Misses
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Activity class="w-3.5 h-3.5 text-amber-400" />
              <span>Signal Sensitivity (d')</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-amber-300">
              {lastSummary.dPrime.toFixed(2)}
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.dPrime > 2.5 ? 'High Discrimination' : 'Normal Attention'}
            </div>
          </div>
        </div>

        <!-- Goblin Reward Banner -->
        <div class="bg-gradient-to-r from-amber-950/30 via-zinc-950 to-zinc-950 border border-amber-500/30 rounded-2xl p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl">👺</span>
            <div>
              <h4 class="text-xs md:text-sm font-bold text-amber-400">
                Goblin Focus Rewards Claimed!
              </h4>
              <p class="text-[11px] text-zinc-400">
                Cognitive telemetry synced to MedGemma clinical readiness engine.
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3 text-xs md:text-sm font-black font-mono">
            <span class="text-emerald-400">+{lastSummary.xpAwarded} XP</span>
            <span class="text-amber-400">+{lastSummary.goldAwarded} Gold</span>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex flex-wrap items-center justify-between gap-3 pt-2">
          <button
            onclick={() => startTest('daily')}
            class="px-5 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs md:text-sm flex items-center gap-2 transition-all"
          >
            <RotateCcw class="w-4 h-4" />
            <span>Retest Cognitive Readiness</span>
          </button>

          <button
            onclick={() => activeTab = 'history'}
            class="px-5 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 font-bold text-xs md:text-sm flex items-center gap-2 transition-all"
          >
            <span>View Historical Logs</span>
            <ArrowRight class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/if}
  {:else if activeTab === 'history'}
    <!-- Historical Sessions List & Time-of-Day Trends -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 space-y-4 shadow-xl">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <History class="w-5 h-5 text-indigo-400" />
          <h3 class="text-base font-bold text-zinc-100">Cognitive Session Log ({recentSessions.length})</h3>
        </div>
        <button
          onclick={() => loadRecentCaritSessions(30)}
          class="text-xs text-zinc-400 hover:text-zinc-200 underline"
        >
          Refresh
        </button>
      </div>

      {#if recentSessions.length === 0}
        <div class="p-8 text-center text-zinc-400 space-y-2">
          <Brain class="w-8 h-8 text-zinc-600 mx-auto" />
          <p class="text-sm font-medium">No cognitive sessions logged yet.</p>
          <p class="text-xs text-zinc-400">Take your first daily CARIT test to start tracking neural readiness.</p>
        </div>
      {:else}
        <div class="space-y-2.5">
          {#each recentSessions as sess (sess.id || sess.timestamp)}
            <div class="p-4 rounded-2xl bg-zinc-950/70 border border-zinc-800/80 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 hover:border-zinc-700 transition-colors">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/30 flex items-center justify-center font-bold text-indigo-400 text-sm">
                  {sess.cognitiveScore}
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold text-zinc-100">{sess.date}</span>
                    <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase {sess.timeOfDay === 'morning' ? 'bg-amber-500/20 text-amber-300' : sess.timeOfDay === 'afternoon' ? 'bg-blue-500/20 text-blue-300' : 'bg-purple-500/20 text-purple-300'}">
                      {sess.timeOfDay}
                    </span>
                    <span class="text-xs text-zinc-400 font-mono">{formatTime(sess.timestamp)}</span>
                  </div>
                  <div class="text-xs text-zinc-400 mt-0.5 flex flex-wrap items-center gap-3">
                    <span>Mean RT: <strong class="text-zinc-200">{sess.meanRtMs.toFixed(0)}ms</strong></span>
                    <span>Inhibition: <strong class="text-emerald-400">{(sess.nogoAccuracy * 100).toFixed(0)}%</strong></span>
                    <span>Go Acc: <strong class="text-zinc-200">{(sess.goAccuracy * 100).toFixed(0)}%</strong></span>
                    <span>d': <strong class="text-amber-300">{sess.dPrime.toFixed(2)}</strong></span>
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2 text-xs font-mono font-bold shrink-0 self-end md:self-center">
                <span class="text-emerald-400">+{sess.xpAwarded} XP</span>
                <span class="text-amber-400">+{sess.goldAwarded} Gold</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else if activeTab === 'rules'}
    <!-- Stimuli & Detailed Task Instructions -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
      <div>
        <h3 class="text-lg font-bold text-zinc-100 flex items-center gap-2">
          <HelpCircle class="w-5 h-5 text-indigo-400" />
          HCP CARIT Stimuli & Experimental Rules
        </h3>
        <p class="text-xs text-zinc-400 mt-1">
          Standardized Cued-Action Response Inhibition Task protocols from the Human Connectome Project.
        </p>
      </div>

      <!-- Shapes Visual Table -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Go Shapes -->
        <div class="p-5 rounded-2xl bg-zinc-950 border border-emerald-500/30 space-y-4">
          <div class="flex items-center justify-between">
            <h4 class="text-sm font-bold text-emerald-400 flex items-center gap-2">
              <CheckCircle2 class="w-4 h-4 text-emerald-400" />
              GO SHAPES (Tap As Fast As Possible)
            </h4>
            <span class="text-[10px] bg-emerald-500/20 text-emerald-300 px-2 py-0.5 rounded-full font-bold">Press</span>
          </div>
          <p class="text-xs text-zinc-400 leading-relaxed">
            When any of these 6 polygon shapes appear on screen for 600ms, tap the button or press Spacebar immediately:
          </p>
          <div class="grid grid-cols-3 gap-3 pt-2">
            {#each ['pentagon', 'hexagon', 'octagon', 'plaque', 'trapezoid', 'parallelogram'] as shape}
              <div class="p-2 rounded-xl bg-zinc-900 border border-zinc-800 flex flex-col items-center gap-1.5">
                <img src={`/carit/${shape}.png`} alt={shape} class="w-12 h-12 object-contain" />
                <span class="text-[10px] font-bold text-zinc-300 capitalize">{shape}</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- No-Go Shapes -->
        <div class="p-5 rounded-2xl bg-zinc-950 border border-rose-500/30 space-y-4">
          <div class="flex items-center justify-between">
            <h4 class="text-sm font-bold text-rose-400 flex items-center gap-2">
              <XCircle class="w-4 h-4 text-rose-400" />
              NO-GO SHAPES (Do NOT Tap)
            </h4>
            <span class="text-[10px] bg-rose-500/20 text-rose-300 px-2 py-0.5 rounded-full font-bold">Withhold</span>
          </div>
          <p class="text-xs text-zinc-400 leading-relaxed">
            When a Circle or Square appears, resist the motor urge to tap. Withhold your press completely:
          </p>
          <div class="grid grid-cols-2 gap-3 pt-2">
            {#each ['circle', 'square'] as shape}
              <div class="p-3 rounded-xl bg-zinc-900 border border-zinc-800 flex flex-col items-center gap-2">
                <img src={`/carit/${shape}.png`} alt={shape} class="w-16 h-16 object-contain" />
                <span class="text-xs font-bold text-rose-300 capitalize">{shape}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
