<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { vismotorState, loadTodayVismotorSummary, loadRecentVismotorSessions, recordVismotorSession } from '$lib/state/vismotor.svelte';
  import { biometricsState, loadLatestBiometrics } from '$lib/state/biometrics.svelte';
  import { goblinState, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { VismotorEngine, type VismotorEngineState } from '$lib/services/vismotorEngine';
  import { 
    VISMOTOR_DAILY_CONDITIONS, 
    VISMOTOR_PRACTICE_CONDITIONS, 
    type VismotorTrialDef, 
    type TargetSide 
  } from '$lib/data/vismotorConditions';
  import type { VismotorTrialRecord } from '$lib/types/vismotor';
  import { 
    Zap, 
    Target, 
    History, 
    Play, 
    RotateCcw, 
    CheckCircle2, 
    XCircle, 
    ArrowRight, 
    Clock, 
    Activity, 
    HelpCircle, 
    Layers,
    ArrowLeft,
    Compass
  } from 'lucide-svelte';

  let engine: VismotorEngine | null = null;
  let activeTab = $state<'test' | 'history' | 'rules'>('test');
  let selectedMode = $state<'daily' | 'practice' | 'full_scan'>('daily');

  // Engine state
  let engineState = $state<VismotorEngineState>('IDLE');
  let countdownNumber = $state<number>(3);
  let currentTrial = $state<VismotorTrialDef | null>(null);
  let currentTrialIndex = $state<number>(0);
  let totalTrialsCount = $state<number>(0);
  let lastTrialRecord = $state<VismotorTrialRecord | null>(null);

  // Active touch feedback
  let isLeftPressed = $state(false);
  let isRightPressed = $state(false);

  let todaySummary = $derived(vismotorState.todaySummary);
  let recentSessions = $derived(vismotorState.recentSessions);
  let lastSummary = $derived(vismotorState.activeSessionResult);

  onMount(async () => {
    await Promise.all([
      loadTodayVismotorSummary(),
      loadRecentVismotorSessions(20),
      loadLatestBiometrics(),
      loadGoblinProfile(),
    ]);

    engine = new VismotorEngine({
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
      onTrialEnd: (record) => {
        lastTrialRecord = record;
      },
      onFinished: async (summary) => {
        await recordVismotorSession(summary);
      },
    });

    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    if (engine) engine.stop();
    window.removeEventListener('keydown', handleKeyDown);
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft' || e.key === 'a' || e.key === 'A') {
      e.preventDefault();
      triggerSideAction('left');
    } else if (e.key === 'ArrowRight' || e.key === 'd' || e.key === 'D') {
      e.preventDefault();
      triggerSideAction('right');
    }
  }

  function triggerSideAction(side: TargetSide) {
    if (!engine) return;
    if (side === 'left') {
      isLeftPressed = true;
      setTimeout(() => { isLeftPressed = false; }, 120);
    } else {
      isRightPressed = true;
      setTimeout(() => { isRightPressed = false; }, 120);
    }
    engine.handleUserAction(side);
  }

  function startTest(mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    selectedMode = mode;
    lastTrialRecord = null;
    vismotorState.activeSessionResult = null;

    let conditions = mode === 'practice' ? VISMOTOR_PRACTICE_CONDITIONS : VISMOTOR_DAILY_CONDITIONS;
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
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-cyan-950/40 border border-cyan-500/20 rounded-3xl p-5 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-cyan-500/20 border border-cyan-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(6,182,212,0.3)]">
        ⚡
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          VISMOTOR Arena
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">
          Daily Visuomotor Choice Reaction Speed & Spatial Motor Coordination (HCP Standard).
        </p>
      </div>
    </div>

    <!-- Quick Stat Summary -->
    <div class="flex items-center gap-3 bg-zinc-950/80 p-3 rounded-2xl border border-zinc-800 shrink-0">
      <div class="flex items-center gap-2 text-cyan-400 font-bold text-xs md:text-sm">
        <Compass class="w-4 h-4" />
        {#if todaySummary?.hasCompletedToday}
          <span class="text-emerald-400 font-black font-mono">Score {todaySummary.latestScore}/100</span>
          <span class="text-[10px] text-zinc-400 font-normal">({todaySummary.latestMeanRtMs?.toFixed(0)}ms Choice RT)</span>
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
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'test' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <Play class="w-4 h-4" />
      <span>Visuomotor Test</span>
    </button>
    <button
      onclick={() => activeTab = 'history'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'history' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <History class="w-4 h-4" />
      <span>History & Logs</span>
    </button>
    <button
      onclick={() => activeTab = 'rules'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'rules' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <HelpCircle class="w-4 h-4" />
      <span>Task Rules</span>
    </button>
  </div>

  {#if activeTab === 'test'}
    {#if engineState === 'IDLE' && !lastSummary}
      <!-- Start Screen -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 text-center shadow-xl">
        <div class="max-w-xl mx-auto space-y-3">
          <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-xs font-bold">
            <Zap class="w-3.5 h-3.5" />
            Visuomotor Choice Reaction
          </div>
          <h2 class="text-2xl md:text-3xl font-black text-zinc-100">
            Daily Visuomotor Speed Test
          </h2>
          <p class="text-xs md:text-sm text-zinc-400 leading-relaxed">
            Measure visual processing latency, spatial choice reaction time, and left vs right hemispheric motor symmetry.
          </p>
        </div>

        <!-- Rules Visual -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 max-w-2xl mx-auto text-left">
          <div class="p-4 rounded-2xl bg-zinc-950 border border-cyan-500/30 flex items-center gap-3.5">
            <div class="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center font-black text-cyan-400 text-lg">
              ◀
            </div>
            <div>
              <h3 class="text-sm font-bold text-cyan-400">LEFT TARGET</h3>
              <p class="text-[11px] text-zinc-300 mt-0.5">Tap LEFT side (or press Left Arrow / A)</p>
            </div>
          </div>

          <div class="p-4 rounded-2xl bg-zinc-950 border border-cyan-500/30 flex items-center gap-3.5">
            <div class="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center font-black text-cyan-400 text-lg">
              ▶
            </div>
            <div>
              <h3 class="text-sm font-bold text-cyan-400">RIGHT TARGET</h3>
              <p class="text-[11px] text-zinc-300 mt-0.5">Tap RIGHT side (or press Right Arrow / D)</p>
            </div>
          </div>
        </div>

        <!-- Launch Buttons -->
        <div class="pt-4 flex flex-col sm:flex-row items-center justify-center gap-3 max-w-lg mx-auto">
          <button
            onclick={() => startTest('daily')}
            class="w-full sm:w-auto flex-1 py-3.5 px-6 rounded-2xl bg-cyan-600 hover:bg-cyan-500 active:scale-[0.98] text-white font-black text-sm md:text-base flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(6,182,212,0.4)] transition-all"
          >
            <Play class="w-5 h-5 fill-current" />
            <span>Start Daily Test (~60s)</span>
          </button>

          <button
            onclick={() => startTest('practice')}
            class="w-full sm:w-auto py-3.5 px-5 rounded-2xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-bold text-xs md:text-sm flex items-center justify-center gap-2 transition-all border border-zinc-700"
          >
            <span>Practice Run (8 trials)</span>
          </button>
        </div>
      </div>
    {:else if engineState !== 'IDLE' && engineState !== 'FINISHED'}
      <!-- Active Arena Viewport -->
      <div class="relative bg-zinc-950 border border-zinc-800 rounded-3xl overflow-hidden shadow-2xl flex flex-col items-center justify-between min-h-[460px] md:min-h-[520px] p-6 select-none">
        
        <!-- Header -->
        <div class="w-full flex items-center justify-between z-10">
          <div class="flex items-center gap-2">
            <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
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

        <!-- Center Viewport with Visual Stimulus -->
        <div class="flex-1 flex flex-col items-center justify-center w-full my-4 relative">
          {#if engineState === 'COUNTDOWN'}
            <div class="flex flex-col items-center justify-center animate-pulse">
              <span class="text-7xl md:text-8xl font-black font-mono text-cyan-400 drop-shadow-[0_0_30px_rgba(6,182,212,0.6)]">
                {countdownNumber}
              </span>
              <span class="text-xs text-zinc-400 font-bold mt-2 tracking-widest uppercase">Get Ready...</span>
            </div>
          {:else if engineState === 'TARGET' && currentTrial}
            <!-- Target Wedge Display (500ms) -->
            <div class="relative flex items-center justify-center w-72 h-72 md:w-80 md:h-80">
              <img
                src={`/vismotor/${currentTrial.targetImage}`}
                alt={`Target ${currentTrial.side}`}
                class="w-full h-full object-contain filter drop-shadow-[0_0_25px_rgba(6,182,212,0.4)]"
              />
            </div>
          {:else if engineState === 'FIXATION'}
            <!-- Fixation Cross (+) -->
            <div class="flex items-center justify-center w-72 h-72 md:w-80 md:h-80">
              <span class="text-6xl md:text-7xl font-mono text-zinc-300 font-bold opacity-80 select-none">
                +
              </span>
            </div>
          {/if}
        </div>

        <!-- Dual Touch Pads (Left / Right) -->
        <div class="w-full grid grid-cols-2 gap-4 z-10">
          <button
            onpointerdown={(e) => { e.preventDefault(); triggerSideAction('left'); }}
            class="py-5 rounded-2xl font-black text-base md:text-lg flex items-center justify-center gap-2 border transition-all active:scale-[0.98] {isLeftPressed ? 'bg-cyan-500 text-zinc-950 border-cyan-400 shadow-[0_0_25px_rgba(6,182,212,0.7)]' : 'bg-zinc-900 hover:bg-zinc-800 text-zinc-100 border-zinc-700 shadow-md'}"
          >
            <ArrowLeft class="w-5 h-5" />
            <span>LEFT (A / ◀)</span>
          </button>

          <button
            onpointerdown={(e) => { e.preventDefault(); triggerSideAction('right'); }}
            class="py-5 rounded-2xl font-black text-base md:text-lg flex items-center justify-center gap-2 border transition-all active:scale-[0.98] {isRightPressed ? 'bg-cyan-500 text-zinc-950 border-cyan-400 shadow-[0_0_25px_rgba(6,182,212,0.7)]' : 'bg-zinc-900 hover:bg-zinc-800 text-zinc-100 border-zinc-700 shadow-md'}"
          >
            <span>RIGHT (D / ▶)</span>
            <ArrowRight class="w-5 h-5" />
          </button>
        </div>
      </div>
    {:else if lastSummary}
      <!-- Results Summary -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
        <div class="flex flex-col md:flex-row items-center justify-between gap-4 border-b border-zinc-800 pb-5">
          <div class="flex items-center gap-3.5">
            <div class="w-14 h-14 rounded-2xl bg-cyan-500/20 border border-cyan-500/40 flex items-center justify-center text-3xl shadow-[0_0_20px_rgba(6,182,212,0.3)]">
              ⚡
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl md:text-2xl font-black text-zinc-100">
                  Visuomotor Performance Assessment
                </h2>
                <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-cyan-500/20 text-cyan-300">
                  {lastSummary.mode === 'daily' ? 'Daily Check-in' : lastSummary.mode}
                </span>
              </div>
              <p class="text-xs text-zinc-400 mt-0.5">
                Completed at {formatTime(lastSummary.timestamp)} • Evaluated against HCP visuomotor speed norms
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3 bg-zinc-950 p-3 rounded-2xl border border-zinc-800">
            <div class="text-right">
              <span class="text-[10px] uppercase font-bold text-zinc-400 block">Visuomotor Score</span>
              <span class="text-2xl font-black font-mono text-emerald-400">{lastSummary.vismotorScore}/100</span>
            </div>
          </div>
        </div>

        <!-- 4 Metric Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4">
          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Clock class="w-3.5 h-3.5 text-cyan-400" />
              <span>Choice RT</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {lastSummary.meanRtMs.toFixed(0)} <span class="text-xs font-normal text-zinc-400">ms</span>
            </div>
            <div class="text-[10px] text-zinc-400">
              Left: {lastSummary.meanLeftRtMs.toFixed(0)}ms • Right: {lastSummary.meanRightRtMs.toFixed(0)}ms
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Target class="w-3.5 h-3.5 text-emerald-400" />
              <span>Spatial Accuracy</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-emerald-400">
              {(lastSummary.accuracy * 100).toFixed(0)}%
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.correctCount} / {lastSummary.totalTrials} Correct Choices
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Compass class="w-3.5 h-3.5 text-amber-400" />
              <span>Motor Symmetry</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-amber-300">
              {lastSummary.hemisphericDifferenceMs.toFixed(0)} <span class="text-xs font-normal text-zinc-400">ms Δ</span>
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.hemisphericDifferenceMs < 40 ? 'Optimal Bilateral Symmetry' : 'Mild Hemispheric Asymmetry'}
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Activity class="w-3.5 h-3.5 text-indigo-400" />
              <span>Consistency</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              ±{lastSummary.rtStdDevMs} <span class="text-xs font-normal text-zinc-400">ms</span>
            </div>
            <div class="text-[10px] text-zinc-400">
              RT Standard Deviation
            </div>
          </div>
        </div>

        <!-- Reward Banner -->
        <div class="bg-gradient-to-r from-cyan-950/30 via-zinc-950 to-zinc-950 border border-cyan-500/30 rounded-2xl p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl">👺</span>
            <div>
              <h4 class="text-xs md:text-sm font-bold text-cyan-400">
                Visuomotor XP & Gold Claimed!
              </h4>
              <p class="text-[11px] text-zinc-400">
                Spatial reaction telemetry synced to MedGemma clinical AI engine.
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3 text-xs md:text-sm font-black font-mono">
            <span class="text-emerald-400">+{lastSummary.xpAwarded} XP</span>
            <span class="text-amber-400">+{lastSummary.goldAwarded} Gold</span>
          </div>
        </div>

        <div class="flex flex-wrap items-center justify-between gap-3 pt-2">
          <button
            onclick={() => startTest('daily')}
            class="px-5 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs md:text-sm flex items-center gap-2 transition-all"
          >
            <RotateCcw class="w-4 h-4" />
            <span>Retest Visuomotor Speed</span>
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
    <!-- History -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 space-y-4 shadow-xl">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <History class="w-5 h-5 text-cyan-400" />
          <h3 class="text-base font-bold text-zinc-100">Visuomotor Session Log ({recentSessions.length})</h3>
        </div>
        <button
          onclick={() => loadRecentVismotorSessions(30)}
          class="text-xs text-zinc-400 hover:text-zinc-200 underline"
        >
          Refresh
        </button>
      </div>

      {#if recentSessions.length === 0}
        <div class="p-8 text-center text-zinc-400 space-y-2">
          <Compass class="w-8 h-8 text-zinc-600 mx-auto" />
          <p class="text-sm font-medium">No visuomotor sessions logged yet.</p>
          <p class="text-xs text-zinc-400">Complete your first daily VISMOTOR test to start tracking choice reaction latency.</p>
        </div>
      {:else}
        <div class="space-y-2.5">
          {#each recentSessions as sess (sess.id || sess.timestamp)}
            <div class="p-4 rounded-2xl bg-zinc-950/70 border border-zinc-800/80 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 hover:border-zinc-700 transition-colors">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center font-bold text-cyan-400 text-sm">
                  {sess.vismotorScore}
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold text-zinc-100">{sess.date}</span>
                    <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase bg-cyan-500/20 text-cyan-300">
                      {sess.timeOfDay}
                    </span>
                    <span class="text-xs text-zinc-400 font-mono">{formatTime(sess.timestamp)}</span>
                  </div>
                  <div class="text-xs text-zinc-400 mt-0.5 flex flex-wrap items-center gap-3">
                    <span>Choice RT: <strong class="text-zinc-200">{sess.meanRtMs.toFixed(0)}ms</strong></span>
                    <span>Acc: <strong class="text-emerald-400">{(sess.accuracy * 100).toFixed(0)}%</strong> ({sess.correctCount}/{sess.totalTrials})</span>
                    <span>Symmetry Δ: <strong class="text-amber-300">{sess.hemisphericDifferenceMs.toFixed(0)}ms</strong></span>
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
    <!-- Rules -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
      <div>
        <h3 class="text-lg font-bold text-zinc-100 flex items-center gap-2">
          <HelpCircle class="w-5 h-5 text-cyan-400" />
          HCP Visuomotor Speed Protocols
        </h3>
        <p class="text-xs text-zinc-400 mt-1">
          Sensorimotor spatial mapping and bilateral reaction speed guidelines.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="p-4 rounded-2xl bg-zinc-950 border border-zinc-800 space-y-2">
          <h4 class="text-sm font-bold text-cyan-400">Left Spatial Target</h4>
          <p class="text-xs text-zinc-400">When high-contrast squares appear on the left, tap Left immediately.</p>
          <img src="/vismotor/leftSquares.png" alt="Left Target" class="w-48 h-48 object-contain mx-auto" />
        </div>

        <div class="p-4 rounded-2xl bg-zinc-950 border border-zinc-800 space-y-2">
          <h4 class="text-sm font-bold text-cyan-400">Right Spatial Target</h4>
          <p class="text-xs text-zinc-400">When high-contrast squares appear on the right, tap Right immediately.</p>
          <img src="/vismotor/rightSquares.png" alt="Right Target" class="w-48 h-48 object-contain mx-auto" />
        </div>
      </div>
    </div>
  {/if}
</div>
