<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { facenameState, loadTodayFaceNameSummary, loadRecentFaceNameSessions, recordFaceNameSession } from '$lib/state/facename.svelte';
  import { biometricsState, loadLatestBiometrics } from '$lib/state/biometrics.svelte';
  import { goblinState, loadGoblinProfile } from '$lib/state/goblin.svelte';
  import { FaceNameEngine, type FaceEngineState } from '$lib/services/facenameEngine';
  import { 
    PRACTICE_FACES, 
    DAILY_MALE_FACES, 
    DAILY_FEMALE_FACES, 
    type FaceItem, 
    type DistractorTrialDef 
  } from '$lib/data/facenameConditions';
  import { 
    Brain, 
    Sparkles, 
    Zap, 
    Target, 
    ShieldAlert, 
    History, 
    Play, 
    RotateCcw, 
    CheckCircle2, 
    XCircle, 
    ArrowRight, 
    Clock, 
    HelpCircle, 
    Users,
    Flame
  } from 'lucide-svelte';

  let engine: FaceNameEngine | null = null;
  let activeTab = $state<'test' | 'history' | 'rules'>('test');
  let selectedMode = $state<'daily' | 'practice' | 'full_scan'>('daily');

  // Engine state
  let engineState = $state<FaceEngineState>('IDLE');
  let countdownNumber = $state<number>(3);
  let cueTitle = $state<string>('');
  let cueSubtitle = $state<string>('');

  // Active items
  let activeFace = $state<FaceItem | null>(null);
  let activeDistractor = $state<DistractorTrialDef | null>(null);
  let activeRecallOptions = $state<string[]>([]);
  let currentItemNumber = $state<number>(0);
  let totalItemsCount = $state<number>(0);

  // Feedback during practice
  let feedbackMessage = $state<string | null>(null);
  let feedbackTimer: any = null;
  let isActionActive = $state(false);

  let todaySummary = $derived(facenameState.todaySummary);
  let recentSessions = $derived(facenameState.recentSessions);
  let lastSummary = $derived(facenameState.activeSessionResult);

  onMount(async () => {
    await Promise.all([
      loadTodayFaceNameSummary(),
      loadRecentFaceNameSessions(20),
      loadLatestBiometrics(),
      loadGoblinProfile(),
    ]);

    engine = new FaceNameEngine({
      onStateChange: (state) => {
        engineState = state;
      },
      onCountdownTick: (count) => {
        countdownNumber = count;
      },
      onCueChange: (title, sub) => {
        cueTitle = title;
        cueSubtitle = sub;
      },
      onMemorizeFace: (face, index, total) => {
        activeFace = face;
        currentItemNumber = index;
        totalItemsCount = total;
      },
      onDistractorStart: (trial, index, total) => {
        activeDistractor = trial;
        currentItemNumber = index;
        totalItemsCount = total;
      },
      onRecallFace: (face, options, index, total) => {
        activeFace = face;
        activeRecallOptions = options;
        currentItemNumber = index;
        totalItemsCount = total;
      },
      onRecallFeedback: (isCorrect, correctName, rt) => {
        if (selectedMode === 'practice') {
          if (feedbackTimer) clearTimeout(feedbackTimer);
          feedbackMessage = isCorrect ? `✅ Correct! (${rt}ms)` : `❌ Incorrect! That was ${correctName}`;
          feedbackTimer = setTimeout(() => { feedbackMessage = null; }, 800);
        }
      },
      onFinished: async (summary) => {
        await recordFaceNameSession(summary);
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
      if (engineState === 'DISTRACTOR_SHAPE' || engineState === 'DISTRACTOR_ISI') {
        e.preventDefault();
        triggerDistractorAction();
      }
    }
  }

  function triggerDistractorAction() {
    if (!engine) return;
    isActionActive = true;
    setTimeout(() => { isActionActive = false; }, 120);
    engine.handleDistractorAction();
  }

  function handleOptionSelect(chosenName: string) {
    if (!engine) return;
    engine.handleRecallSelection(chosenName);
  }

  function startTest(mode: 'daily' | 'practice' | 'full_scan' = 'daily') {
    selectedMode = mode;
    feedbackMessage = null;
    facenameState.activeSessionResult = null;
    engine?.start(mode);
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
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-purple-950/40 border border-purple-500/20 rounded-3xl p-5 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-purple-500/20 border border-purple-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(168,85,247,0.3)]">
        🧩
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          FACENAME Memory Arena
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">
          Daily Associative Long-Term Memory & Face-Name Recognition (HCP Standard).
        </p>
      </div>
    </div>

    <!-- Quick Stat Summary -->
    <div class="flex items-center gap-3 bg-zinc-950/80 p-3 rounded-2xl border border-zinc-800 shrink-0">
      <div class="flex items-center gap-2 text-purple-400 font-bold text-xs md:text-sm">
        <Brain class="w-4 h-4" />
        {#if todaySummary?.hasCompletedToday}
          <span class="text-emerald-400 font-black font-mono">Score {todaySummary.latestScore}/100</span>
          <span class="text-[10px] text-zinc-400 font-normal">({((todaySummary.latestRecallAccuracy || 0) * 100).toFixed(0)}% Recall)</span>
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
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'test' ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <Play class="w-4 h-4" />
      <span>Memory Test</span>
    </button>
    <button
      onclick={() => activeTab = 'history'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'history' ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <History class="w-4 h-4" />
      <span>History & Logs</span>
    </button>
    <button
      onclick={() => activeTab = 'rules'}
      class="px-4 py-2 rounded-xl text-xs md:text-sm font-bold flex items-center gap-2 transition-all {activeTab === 'rules' ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <HelpCircle class="w-4 h-4" />
      <span>Faces & Rules</span>
    </button>
  </div>

  {#if activeTab === 'test'}
    {#if engineState === 'IDLE' && !lastSummary}
      <!-- Start Screen -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 text-center shadow-xl">
        <div class="max-w-xl mx-auto space-y-3">
          <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-purple-500/10 border border-purple-500/30 text-purple-400 text-xs font-bold">
            <Sparkles class="w-3.5 h-3.5" />
            Hippocampal Associative Memory
          </div>
          <h2 class="text-2xl md:text-3xl font-black text-zinc-100">
            Daily Face-Name Memory Test
          </h2>
          <p class="text-xs md:text-sm text-zinc-400 leading-relaxed">
            Assess your long-term memory encoding, distractor resistance, and associative retrieval speed.
          </p>
        </div>

        <!-- 3-Phase Workflow Explanation -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 max-w-3xl mx-auto text-left">
          <div class="p-4 rounded-2xl bg-zinc-950/70 border border-purple-500/20 space-y-1.5">
            <div class="flex items-center gap-2 text-purple-400 font-bold text-xs">
              <span class="w-5 h-5 rounded-full bg-purple-500/20 flex items-center justify-center text-[10px]">1</span>
              <span>MEMORIZE (4s/face)</span>
            </div>
            <p class="text-[11px] text-zinc-400">Study each face carefully and remember their associated name.</p>
          </div>

          <div class="p-4 rounded-2xl bg-zinc-950/70 border border-amber-500/20 space-y-1.5">
            <div class="flex items-center gap-2 text-amber-400 font-bold text-xs">
              <span class="w-5 h-5 rounded-full bg-amber-500/20 flex items-center justify-center text-[10px]">2</span>
              <span>DISTRACTOR GAME</span>
            </div>
            <p class="text-[11px] text-zinc-400">Tap polygon shapes, withhold on circle/square to clear working memory buffer.</p>
          </div>

          <div class="p-4 rounded-2xl bg-zinc-950/70 border border-emerald-500/20 space-y-1.5">
            <div class="flex items-center gap-2 text-emerald-400 font-bold text-xs">
              <span class="w-5 h-5 rounded-full bg-emerald-500/20 flex items-center justify-center text-[10px]">3</span>
              <span>RECALL QUIZ</span>
            </div>
            <p class="text-[11px] text-zinc-400">Choose the correct name for each face as fast and accurately as possible.</p>
          </div>
        </div>

        <!-- Launch Buttons -->
        <div class="pt-4 flex flex-col sm:flex-row items-center justify-center gap-3 max-w-lg mx-auto">
          <button
            onclick={() => startTest('daily')}
            class="w-full sm:w-auto flex-1 py-3.5 px-6 rounded-2xl bg-purple-600 hover:bg-purple-500 active:scale-[0.98] text-white font-black text-sm md:text-base flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(168,85,247,0.4)] transition-all"
          >
            <Play class="w-5 h-5 fill-current" />
            <span>Start Daily Test (~90s)</span>
          </button>

          <button
            onclick={() => startTest('practice')}
            class="w-full sm:w-auto py-3.5 px-5 rounded-2xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-bold text-xs md:text-sm flex items-center justify-center gap-2 transition-all border border-zinc-700"
          >
            <span>Practice Run (3 faces)</span>
          </button>
        </div>
      </div>
    {:else if engineState !== 'IDLE' && engineState !== 'FINISHED'}
      <!-- Active In-Game Arena -->
      <div class="relative bg-zinc-950 border border-zinc-800 rounded-3xl overflow-hidden shadow-2xl flex flex-col items-center justify-between min-h-[460px] md:min-h-[520px] p-6 select-none">
        
        <!-- Header Progress -->
        <div class="w-full flex items-center justify-between z-10">
          <div class="flex items-center gap-2">
            <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30">
              {engineState} • {currentItemNumber} / {totalItemsCount || 1}
            </span>
          </div>

          <button
            onclick={stopTest}
            class="text-xs px-3 py-1 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800 transition-colors"
          >
            Cancel
          </button>
        </div>

        <!-- Center Viewport -->
        <div class="flex-1 flex flex-col items-center justify-center w-full my-4 relative">
          {#if engineState === 'COUNTDOWN'}
            <div class="flex flex-col items-center justify-center animate-pulse">
              <span class="text-7xl md:text-8xl font-black font-mono text-purple-400 drop-shadow-[0_0_30px_rgba(168,85,247,0.6)]">
                {countdownNumber}
              </span>
              <span class="text-xs text-zinc-400 font-bold mt-2 tracking-widest uppercase">Get Ready...</span>
            </div>
          {:else if engineState === 'CUE'}
            <div class="text-center space-y-2 max-w-md animate-fade-in">
              <h3 class="text-xl md:text-2xl font-black text-purple-300 tracking-wider">
                {cueTitle}
              </h3>
              <p class="text-xs text-zinc-400 leading-relaxed">
                {cueSubtitle}
              </p>
            </div>
          {:else if engineState === 'MEMORIZE' && activeFace}
            <!-- Memorize Portrait & Name -->
            <div class="flex flex-col items-center gap-4 animate-fade-in">
              <div class="w-48 h-48 md:w-60 md:h-60 rounded-3xl overflow-hidden border-2 border-purple-500/50 shadow-[0_0_30px_rgba(168,85,247,0.3)] bg-zinc-900">
                <img
                  src={`/facename/${activeFace.image}`}
                  alt={activeFace.name}
                  class="w-full h-full object-cover"
                />
              </div>
              <div class="px-6 py-2 rounded-2xl bg-purple-500/20 border border-purple-500/40 text-lg md:text-xl font-black text-zinc-100 font-mono tracking-wide shadow-md">
                {activeFace.name}
              </div>
            </div>
          {:else if (engineState === 'DISTRACTOR_SHAPE' || engineState === 'DISTRACTOR_ISI') && activeDistractor}
            <!-- Distractor Shape / Fixation -->
            <div class="flex flex-col items-center justify-center w-56 h-56 md:w-64 md:h-64">
              {#if engineState === 'DISTRACTOR_SHAPE'}
                <img
                  src={`/facename/${activeDistractor.stim}`}
                  alt={activeDistractor.shapeName}
                  class="w-full h-full object-contain filter drop-shadow-[0_0_20px_rgba(255,255,255,0.2)]"
                />
              {:else}
                <span class="text-6xl md:text-7xl font-mono text-zinc-300 font-bold opacity-80">+</span>
              {/if}
            </div>
          {:else if engineState === 'RECALL' && activeFace}
            <!-- Recall Recognition Phase -->
            <div class="flex flex-col items-center gap-4 w-full max-w-md animate-fade-in">
              <div class="w-40 h-40 md:w-48 md:h-48 rounded-3xl overflow-hidden border-2 border-purple-500/40 shadow-xl bg-zinc-900">
                <img
                  src={`/facename/${activeFace.image}`}
                  alt="Who is this?"
                  class="w-full h-full object-cover"
                />
              </div>

              <span class="text-xs text-zinc-400 font-bold uppercase tracking-wider">Who is this person?</span>

              <!-- 4 Multiple Choice Buttons -->
              <div class="grid grid-cols-2 gap-3 w-full">
                {#each activeRecallOptions as opt}
                  <button
                    onclick={() => handleOptionSelect(opt)}
                    class="py-3 px-4 rounded-2xl bg-zinc-900 hover:bg-purple-600 active:scale-[0.98] border border-zinc-700 hover:border-purple-400 text-zinc-100 font-bold text-sm md:text-base transition-all shadow-md"
                  >
                    {opt}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Practice Feedback -->
          {#if feedbackMessage}
            <div class="absolute bottom-2 px-4 py-1.5 rounded-full bg-zinc-900/90 border border-purple-500/40 text-xs font-bold text-zinc-100 backdrop-blur-md shadow-lg animate-bounce">
              {feedbackMessage}
            </div>
          {/if}
        </div>

        <!-- Distractor Tap Pad if active -->
        {#if engineState === 'DISTRACTOR_SHAPE' || engineState === 'DISTRACTOR_ISI'}
          <div class="w-full flex flex-col items-center gap-2 z-10">
            <button
              onpointerdown={(e) => { e.preventDefault(); triggerDistractorAction(); }}
              class="w-full py-4 rounded-2xl font-black text-sm md:text-base flex items-center justify-center gap-2 border transition-all active:scale-[0.98] {isActionActive ? 'bg-emerald-500 text-zinc-950 border-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.7)]' : 'bg-zinc-900 hover:bg-zinc-800 text-zinc-100 border-zinc-700 shadow-md'}"
            >
              <Zap class="w-4 h-4" />
              <span>TAP ON POLYGONS (Withhold on Circle/Square)</span>
            </button>
          </div>
        {/if}
      </div>
    {:else if lastSummary}
      <!-- Results & Metrics -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
        <div class="flex flex-col md:flex-row items-center justify-between gap-4 border-b border-zinc-800 pb-5">
          <div class="flex items-center gap-3.5">
            <div class="w-14 h-14 rounded-2xl bg-purple-500/20 border border-purple-500/40 flex items-center justify-center text-3xl shadow-[0_0_20px_rgba(168,85,247,0.3)]">
              🧩
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl md:text-2xl font-black text-zinc-100">
                  Associative Memory Assessment
                </h2>
                <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-purple-500/20 text-purple-300">
                  {lastSummary.mode === 'daily' ? 'Daily Check-in' : lastSummary.mode}
                </span>
              </div>
              <p class="text-xs text-zinc-400 mt-0.5">
                Completed at {formatTime(lastSummary.timestamp)} • Evaluated against HCP associative retention norms
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3 bg-zinc-950 p-3 rounded-2xl border border-zinc-800">
            <div class="text-right">
              <span class="text-[10px] uppercase font-bold text-zinc-400 block">Memory Readiness</span>
              <span class="text-2xl font-black font-mono text-emerald-400">{lastSummary.memoryScore}/100</span>
            </div>
          </div>
        </div>

        <!-- 4 Metric Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4">
          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
              <span>Recall Accuracy</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-emerald-400">
              {(lastSummary.recallAccuracy * 100).toFixed(0)}%
            </div>
            <div class="text-[10px] text-zinc-400">
              {lastSummary.correctRecallCount} / {lastSummary.totalRecalled} Correct Names
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Clock class="w-3.5 h-3.5 text-purple-400" />
              <span>Retrieval Latency</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {lastSummary.meanRecallRtMs.toFixed(0)} <span class="text-xs font-normal text-zinc-400">ms</span>
            </div>
            <div class="text-[10px] text-zinc-400">
              Median: {lastSummary.medianRecallRtMs}ms
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <ShieldAlert class="w-3.5 h-3.5 text-amber-400" />
              <span>Distractor Resistance</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {(lastSummary.distractorAccuracy * 100).toFixed(0)}%
            </div>
            <div class="text-[10px] text-zinc-400">
              Working Memory Delay
            </div>
          </div>

          <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-1">
            <div class="flex items-center gap-1.5 text-xs text-zinc-400 font-semibold">
              <Users class="w-3.5 h-3.5 text-indigo-400" />
              <span>Faces Encoded</span>
            </div>
            <div class="text-xl md:text-2xl font-black font-mono text-zinc-100">
              {lastSummary.totalMemorized}
            </div>
            <div class="text-[10px] text-zinc-400">
              Total Pairs Tested
            </div>
          </div>
        </div>

        <!-- Reward Banner -->
        <div class="bg-gradient-to-r from-purple-950/30 via-zinc-950 to-zinc-950 border border-purple-500/30 rounded-2xl p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl">👺</span>
            <div>
              <h4 class="text-xs md:text-sm font-bold text-purple-400">
                Memory XP & Gold Claimed!
              </h4>
              <p class="text-[11px] text-zinc-400">
                Associative retention telemetry synced to MedGemma clinical AI.
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
            class="px-5 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-bold text-xs md:text-sm flex items-center gap-2 transition-all"
          >
            <RotateCcw class="w-4 h-4" />
            <span>Retest Face-Name Memory</span>
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
    <!-- History View -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 space-y-4 shadow-xl">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <History class="w-5 h-5 text-purple-400" />
          <h3 class="text-base font-bold text-zinc-100">Memory Session Log ({recentSessions.length})</h3>
        </div>
        <button
          onclick={() => loadRecentFaceNameSessions(30)}
          class="text-xs text-zinc-400 hover:text-zinc-200 underline"
        >
          Refresh
        </button>
      </div>

      {#if recentSessions.length === 0}
        <div class="p-8 text-center text-zinc-400 space-y-2">
          <Brain class="w-8 h-8 text-zinc-600 mx-auto" />
          <p class="text-sm font-medium">No memory sessions logged yet.</p>
          <p class="text-xs text-zinc-400">Complete your first daily FACENAME test to start tracking recall accuracy.</p>
        </div>
      {:else}
        <div class="space-y-2.5">
          {#each recentSessions as sess (sess.id || sess.timestamp)}
            <div class="p-4 rounded-2xl bg-zinc-950/70 border border-zinc-800/80 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 hover:border-zinc-700 transition-colors">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/30 flex items-center justify-center font-bold text-purple-400 text-sm">
                  {sess.memoryScore}
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold text-zinc-100">{sess.date}</span>
                    <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase bg-purple-500/20 text-purple-300">
                      {sess.timeOfDay}
                    </span>
                    <span class="text-xs text-zinc-400 font-mono">{formatTime(sess.timestamp)}</span>
                  </div>
                  <div class="text-xs text-zinc-400 mt-0.5 flex flex-wrap items-center gap-3">
                    <span>Recall: <strong class="text-emerald-400">{(sess.recallAccuracy * 100).toFixed(0)}%</strong> ({sess.correctRecallCount}/{sess.totalRecalled})</span>
                    <span>Retrieval RT: <strong class="text-zinc-200">{sess.meanRecallRtMs.toFixed(0)}ms</strong></span>
                    <span>Distractor: <strong class="text-zinc-300">{(sess.distractorAccuracy * 100).toFixed(0)}%</strong></span>
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
    <!-- Rules View -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-6 md:p-8 space-y-6 shadow-xl">
      <div>
        <h3 class="text-lg font-bold text-zinc-100 flex items-center gap-2">
          <HelpCircle class="w-5 h-5 text-purple-400" />
          HCP Face-Name Task Guidelines
        </h3>
        <p class="text-xs text-zinc-400 mt-1">
          Hippocampal associative encoding and delayed retrieval assessment protocols.
        </p>
      </div>

      <div class="space-y-4">
        <h4 class="text-sm font-bold text-zinc-200">Stimulus Face Database (Sample Pairs)</h4>
        <div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
          {#each [...DAILY_MALE_FACES, ...DAILY_FEMALE_FACES] as face}
            <div class="p-3 rounded-2xl bg-zinc-950 border border-zinc-800 flex flex-col items-center gap-2">
              <div class="w-16 h-16 rounded-xl overflow-hidden bg-zinc-900 border border-zinc-700">
                <img src={`/facename/${face.image}`} alt={face.name} class="w-full h-full object-cover" />
              </div>
              <span class="text-xs font-black text-purple-300 font-mono">{face.name}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
