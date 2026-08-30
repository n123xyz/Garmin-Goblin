<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    Moon, Sparkles, Brain, Heart, Activity, ShieldCheck, 
    ChevronRight, Calendar, TrendingUp, Clock, Zap, Coffee, 
    Droplets, Sun, Wind, CheckCircle2, AlertCircle, Compass,
    History, Bed, Shield, Eye, Flame
  } from 'lucide-svelte';
  import { biometricsState, loadLatestBiometrics, loadBiometricsHistory } from '$lib/state/biometrics.svelte';
  import { goblinState, setGoblinSpeech } from '$lib/state/goblin.svelte';
  import { 
    sleepState, 
    getHistoricalSleepRecords, 
    calculateSleepAggregates, 
    generateSleepAdvisory,
    parseSleepRecord,
    type SleepRecord
  } from '$lib/state/sleep.svelte';

  let selectedDate = $state<string | null>(null);
  let timeRangeDays = $state(14);

  onMount(async () => {
    await loadLatestBiometrics();
    await loadBiometricsHistory(30);
  });

  let rawRecords = $derived(getHistoricalSleepRecords(timeRangeDays));
  let aggregates = $derived(calculateSleepAggregates(rawRecords));

  let activeRecord = $derived.by<SleepRecord | null>(() => {
    if (selectedDate) {
      const match = rawRecords.find(r => r.date === selectedDate);
      if (match) return match;
    }
    if (rawRecords.length > 0) {
      return rawRecords[0];
    }
    if (biometricsState.current) {
      return parseSleepRecord(biometricsState.current);
    }
    return null;
  });

  let advisory = $derived(generateSleepAdvisory(activeRecord, aggregates, biometricsState.current));

  function getScoreColor(score: number): { text: string; bg: string; border: string; label: string } {
    if (score >= 85) return { text: 'text-emerald-400', bg: 'bg-emerald-500/20', border: 'border-emerald-500/40', label: 'Optimal Recovery' };
    if (score >= 70) return { text: 'text-cyan-400', bg: 'bg-cyan-500/20', border: 'border-cyan-500/40', label: 'Good Sleep' };
    if (score >= 55) return { text: 'text-amber-400', bg: 'bg-amber-500/20', border: 'border-amber-500/40', label: 'Moderate Rest' };
    return { text: 'text-rose-400', bg: 'bg-rose-500/20', border: 'border-rose-500/40', label: 'High Sleep Debt' };
  }

  function formatDayOfWeek(dateStr: string): string {
    try {
      const dt = new Date(dateStr + 'T00:00:00');
      return dt.toLocaleDateString([], { weekday: 'short' });
    } catch {
      return '';
    }
  }
</script>

<div class="max-w-6xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Top Header Strip -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-zinc-800 pb-4">
    <div>
      <div class="flex items-center gap-2.5">
        <div class="w-10 h-10 rounded-2xl bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 flex items-center justify-center text-xl shadow-[0_0_15px_rgba(99,102,241,0.25)]">
          🌙
        </div>
        <h1 class="text-2xl md:text-3xl font-black text-zinc-100">Sleep Architecture & Advisor</h1>
      </div>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        On-device circadian telemetry, multi-day historical sleep stage trends, and clinical recovery coaching.
      </p>
    </div>

    <!-- Time Range Filter & Status -->
    <div class="flex flex-wrap items-center gap-2">
      <div class="flex items-center bg-zinc-900 border border-zinc-800 rounded-2xl p-1 shadow-sm">
        <button
          onclick={() => timeRangeDays = 7}
          class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {timeRangeDays === 7 ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          7 Days
        </button>
        <button
          onclick={() => timeRangeDays = 14}
          class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {timeRangeDays === 14 ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          14 Days
        </button>
        <button
          onclick={() => timeRangeDays = 30}
          class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {timeRangeDays === 30 ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          30 Days
        </button>
      </div>

      <span class="text-xs px-3 py-2 rounded-2xl font-bold bg-zinc-900 text-zinc-300 border border-zinc-800 flex items-center gap-1.5">
        <ShieldCheck class="w-4 h-4 text-emerald-400" />
        Offline Inference
      </span>
    </div>
  </div>

  <!-- Primary Selected Night Focus Card -->
  {#if activeRecord}
    {@const scoreInfo = getScoreColor(activeRecord.sleepScore)}
    <div class="bg-gradient-to-br from-zinc-900 via-zinc-900 to-indigo-950/30 border border-indigo-500/30 rounded-3xl p-5 md:p-6 shadow-2xl space-y-5">
      <!-- Focus Card Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-4">
        <div>
          <span class="text-[10px] font-bold uppercase tracking-wider text-indigo-400 block">Inspecting Night Partition</span>
          <h2 class="text-xl md:text-2xl font-black text-zinc-100 font-mono flex items-center gap-2">
            <span>{activeRecord.date}</span>
            <span class="text-xs font-bold text-zinc-400 font-sans">({formatDayOfWeek(activeRecord.date)})</span>
            {#if activeRecord.date === biometricsState.current?.date}
              <span class="text-[10px] px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 font-bold">LATEST</span>
            {/if}
          </h2>
        </div>

        {#if selectedDate}
          <button
            onclick={() => selectedDate = null}
            class="text-xs px-3 py-1.5 rounded-xl font-bold bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border border-zinc-700 flex items-center gap-1.5 self-start cursor-pointer transition-colors"
          >
            <Calendar class="w-3.5 h-3.5 text-indigo-400" />
            <span>Reset to Latest</span>
          </button>
        {/if}
      </div>

      <!-- Core Sleep Biometrics Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3.5">
        <!-- Sleep Score -->
        <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800/80 flex flex-col justify-between">
          <div class="flex items-center justify-between">
            <span class="text-[10px] uppercase font-bold text-zinc-400">Sleep Score</span>
            <span class="text-base">🌙</span>
          </div>
          <div class="my-1">
            <span class="text-2xl md:text-3xl font-black font-mono {scoreInfo.text}">{activeRecord.sleepScore}</span>
            <span class="text-xs text-zinc-500 font-mono">/100</span>
          </div>
          <span class="text-[10px] font-bold px-2 py-0.5 rounded-md {scoreInfo.bg} {scoreInfo.text} border {scoreInfo.border} self-start">
            {scoreInfo.label}
          </span>
        </div>

        <!-- Total Sleep Duration -->
        <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800/80 flex flex-col justify-between">
          <div class="flex items-center justify-between">
            <span class="text-[10px] uppercase font-bold text-zinc-400">Total Duration</span>
            <span class="text-base">⏱️</span>
          </div>
          <div class="my-1">
            <span class="text-2xl md:text-3xl font-black font-mono text-zinc-100">{activeRecord.formattedDuration}</span>
          </div>
          <div class="flex items-center justify-between text-[10px] text-zinc-400">
            <span>Sleep Efficiency:</span>
            <strong class="text-zinc-200 font-mono font-bold">{activeRecord.sleepEfficiencyPct}%</strong>
          </div>
        </div>

        <!-- Autonomic HRV -->
        <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800/80 flex flex-col justify-between">
          <div class="flex items-center justify-between">
            <span class="text-[10px] uppercase font-bold text-zinc-400">Overnight HRV</span>
            <span class="text-base">⚡</span>
          </div>
          <div class="my-1">
            <span class="text-2xl md:text-3xl font-black font-mono text-emerald-400">
              {activeRecord.hrvRmssd > 0 ? activeRecord.hrvRmssd : '--'}
            </span>
            <span class="text-xs text-zinc-500 font-mono">ms RMSSD</span>
          </div>
          <span class="text-[10px] font-bold text-zinc-300">
            Status: <strong class="text-emerald-300">{activeRecord.hrvStatus}</strong>
          </span>
        </div>

        <!-- Basal Resting HR -->
        <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800/80 flex flex-col justify-between">
          <div class="flex items-center justify-between">
            <span class="text-[10px] uppercase font-bold text-zinc-400">Resting Heart Rate</span>
            <span class="text-base">💓</span>
          </div>
          <div class="my-1">
            <span class="text-2xl md:text-3xl font-black font-mono text-rose-400">
              {activeRecord.restingHr > 0 ? activeRecord.restingHr : '--'}
            </span>
            <span class="text-xs text-zinc-500 font-mono">bpm</span>
          </div>
          <span class="text-[10px] text-zinc-400">
            Efficiency: <strong class="text-zinc-200">{activeRecord.sleepEfficiencyPct}%</strong>
          </span>
        </div>
      </div>

      <!-- Sleep Architecture 4-Stage Stacked Visualizer -->
      <div class="space-y-3 bg-zinc-950/70 p-4 md:p-5 rounded-2xl border border-zinc-800/80">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div>
            <h3 class="text-sm font-bold text-zinc-100 flex items-center gap-2">
              <span>🧬</span> Sleep Architecture & Hypnogram Stages
            </h3>
            <span class="text-xs text-zinc-400">Distribution of delta slow-wave, REM, and non-REM restorative sleep cycles.</span>
          </div>
          <div class="flex items-center gap-3 text-[10px] font-mono">
            <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-indigo-500"></span> Deep ({activeRecord.deep.percentage}%)</span>
            <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-purple-500"></span> REM ({activeRecord.rem.percentage}%)</span>
            <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-sky-500"></span> Light ({activeRecord.light.percentage}%)</span>
            <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-rose-500"></span> Awake ({activeRecord.awake.percentage}%)</span>
          </div>
        </div>

        <!-- 4-Stage Color Bar -->
        <div class="w-full h-4 bg-zinc-900 rounded-full overflow-hidden flex border border-zinc-800">
          <div class="h-full bg-indigo-500 transition-all duration-500" style="width: {activeRecord.deep.percentage}%;" title="Deep: {activeRecord.deep.percentage}%"></div>
          <div class="h-full bg-purple-500 transition-all duration-500" style="width: {activeRecord.rem.percentage}%;" title="REM: {activeRecord.rem.percentage}%"></div>
          <div class="h-full bg-sky-500 transition-all duration-500" style="width: {activeRecord.light.percentage}%;" title="Light: {activeRecord.light.percentage}%"></div>
          <div class="h-full bg-rose-500 transition-all duration-500" style="width: {activeRecord.awake.percentage}%;" title="Awake: {activeRecord.awake.percentage}%"></div>
        </div>

        <!-- Detailed Stage Cards -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-2">
          <!-- Deep Sleep -->
          <div class="bg-zinc-900/90 border border-indigo-500/30 rounded-xl p-3 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-indigo-400 flex items-center gap-1">
                <span>⚡</span> Deep (Delta)
              </span>
              <span class="text-xs font-mono font-black text-zinc-100">{activeRecord.deep.hours}h {String(activeRecord.deep.minutes).padStart(2, '0')}m</span>
            </div>
            <div class="flex items-center justify-between text-[10px] text-zinc-400 font-mono">
              <span>{activeRecord.deep.percentage}% of total</span>
              <span class="text-indigo-300 font-bold">{activeRecord.deep.benchmark}</span>
            </div>
            <p class="text-[11px] text-zinc-400 leading-snug">{activeRecord.deep.description}</p>
          </div>

          <!-- REM Sleep -->
          <div class="bg-zinc-900/90 border border-purple-500/30 rounded-xl p-3 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-purple-400 flex items-center gap-1">
                <span>🧠</span> REM (Dream)
              </span>
              <span class="text-xs font-mono font-black text-zinc-100">{activeRecord.rem.hours}h {String(activeRecord.rem.minutes).padStart(2, '0')}m</span>
            </div>
            <div class="flex items-center justify-between text-[10px] text-zinc-400 font-mono">
              <span>{activeRecord.rem.percentage}% of total</span>
              <span class="text-purple-300 font-bold">{activeRecord.rem.benchmark}</span>
            </div>
            <p class="text-[11px] text-zinc-400 leading-snug">{activeRecord.rem.description}</p>
          </div>

          <!-- Light Sleep -->
          <div class="bg-zinc-900/90 border border-sky-500/30 rounded-xl p-3 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-sky-400 flex items-center gap-1">
                <span>💤</span> Light (Core)
              </span>
              <span class="text-xs font-mono font-black text-zinc-100">{activeRecord.light.hours}h {String(activeRecord.light.minutes).padStart(2, '0')}m</span>
            </div>
            <div class="flex items-center justify-between text-[10px] text-zinc-400 font-mono">
              <span>{activeRecord.light.percentage}% of total</span>
              <span class="text-sky-300 font-bold">{activeRecord.light.benchmark}</span>
            </div>
            <p class="text-[11px] text-zinc-400 leading-snug">{activeRecord.light.description}</p>
          </div>

          <!-- Awake -->
          <div class="bg-zinc-900/90 border border-rose-500/30 rounded-xl p-3 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-rose-400 flex items-center gap-1">
                <span>👁️</span> Awake / Alert
              </span>
              <span class="text-xs font-mono font-black text-zinc-100">{activeRecord.awake.hours}h {String(activeRecord.awake.minutes).padStart(2, '0')}m</span>
            </div>
            <div class="flex items-center justify-between text-[10px] text-zinc-400 font-mono">
              <span>{activeRecord.awake.percentage}% of total</span>
              <span class="text-rose-300 font-bold">{activeRecord.awake.benchmark}</span>
            </div>
            <p class="text-[11px] text-zinc-400 leading-snug">{activeRecord.awake.description}</p>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Multi-Night Historical Trend Explorer -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <TrendingUp class="w-5 h-5 text-indigo-400" />
        <h3 class="text-base font-bold text-zinc-100">Multi-Night Historical Sleep Trends ({rawRecords.length} Nights)</h3>
      </div>
      <span class="text-xs text-zinc-400 font-mono">Tap any night to inspect individual telemetry</span>
    </div>

    <!-- Interactive Historical Multi-Night Bars -->
    {#if rawRecords.length > 0}
      <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2.5 pt-2">
        {#each rawRecords as record (record.id)}
          {@const isSelected = (selectedDate === record.date) || (!selectedDate && record.date === biometricsState.current?.date)}
          {@const scoreInfo = getScoreColor(record.sleepScore)}
          <button
            onclick={() => selectedDate = record.date}
            class="p-3 rounded-2xl border text-left flex flex-col justify-between transition-all duration-200 min-h-[145px] cursor-pointer
              {isSelected 
                ? 'bg-indigo-950/50 border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.3)] ring-2 ring-indigo-500' 
                : 'bg-zinc-950/70 border-zinc-800/80 hover:bg-zinc-800/60 hover:border-zinc-700'}"
          >
            <!-- Day Header -->
            <div class="flex items-center justify-between w-full border-b border-zinc-800/60 pb-1.5">
              <div>
                <span class="text-[10px] font-bold uppercase tracking-wider text-zinc-400 block">
                  {formatDayOfWeek(record.date)}
                </span>
                <span class="text-xs font-black font-mono text-zinc-200">
                  {record.date.substring(5)}
                </span>
              </div>

              <span class="text-xs font-black font-mono px-2 py-0.5 rounded-md {scoreInfo.bg} {scoreInfo.text} border {scoreInfo.border}">
                {record.sleepScore}
              </span>
            </div>

            <!-- Mini Hypnogram Bar -->
            <div class="my-2 w-full space-y-1">
              <div class="w-full h-2 rounded-full overflow-hidden flex bg-zinc-900 border border-zinc-800">
                <div class="h-full bg-indigo-500" style="width: {record.deep.percentage}%;"></div>
                <div class="h-full bg-purple-500" style="width: {record.rem.percentage}%;"></div>
                <div class="h-full bg-sky-500" style="width: {record.light.percentage}%;"></div>
                <div class="h-full bg-rose-500" style="width: {record.awake.percentage}%;"></div>
              </div>
              <span class="text-[11px] font-mono font-bold text-zinc-300 block text-center">
                {record.formattedDuration}
              </span>
            </div>

            <!-- Overnight Biometrics Mini Footer -->
            <div class="flex items-center justify-between text-[10px] font-mono text-zinc-400 pt-1.5 border-t border-zinc-800/60 w-full">
              <span>💓 {record.restingHr > 0 ? `${record.restingHr}` : '--'}</span>
              <span>⚡ {record.hrvRmssd > 0 ? `${record.hrvRmssd}ms` : '--'}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <p class="text-xs text-zinc-500 italic">No historical sleep data found yet. Sync your watch to build daily records.</p>
    {/if}

    <!-- Aggregate Performance Summary Strip -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-3">
      <div class="bg-zinc-950/80 p-3 rounded-xl border border-zinc-800/80 text-center">
        <span class="text-[10px] uppercase font-bold text-zinc-400 block">Avg Sleep Score</span>
        <span class="text-lg font-black font-mono text-indigo-400">{aggregates.avgScore}/100</span>
        <span class="text-[10px] text-zinc-500 block">Over {aggregates.totalNights} Nights</span>
      </div>

      <div class="bg-zinc-950/80 p-3 rounded-xl border border-zinc-800/80 text-center">
        <span class="text-[10px] uppercase font-bold text-zinc-400 block">Avg Duration</span>
        <span class="text-lg font-black font-mono text-zinc-200">{aggregates.formattedAvgDuration}</span>
        <span class="text-[10px] text-zinc-500 block">Nightly Baseline</span>
      </div>

      <div class="bg-zinc-950/80 p-3 rounded-xl border border-zinc-800/80 text-center">
        <span class="text-[10px] uppercase font-bold text-zinc-400 block">Restorative Ratio</span>
        <span class="text-lg font-black font-mono text-emerald-400">{aggregates.avgDeepPct + aggregates.avgRemPct}%</span>
        <span class="text-[10px] text-zinc-500 block">Deep + REM Combined</span>
      </div>

      <div class="bg-zinc-950/80 p-3 rounded-xl border border-zinc-800/80 text-center">
        <span class="text-[10px] uppercase font-bold text-zinc-400 block">Sleep Consistency</span>
        <span class="text-lg font-black font-mono text-cyan-400">{aggregates.consistencyScore}%</span>
        <span class="text-[10px] text-zinc-500 block">Circadian Rhythm Index</span>
      </div>
    </div>
  </div>

  <!-- MedGemma Clinical Advisor & Goblin Sleep Coach -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- MedGemma Clinical Sleep Briefing -->
    <div class="lg:col-span-7 bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <div class="flex items-center gap-2">
          <Brain class="w-5 h-5 text-indigo-400" />
          <h3 class="text-base font-bold text-zinc-100">MedGemma Clinical Sleep Briefing</h3>
        </div>
        <span class="text-[10px] px-2.5 py-0.5 rounded-full font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 uppercase tracking-wider">
          On-Device Medical AI
        </span>
      </div>

      <p class="text-xs md:text-sm text-zinc-300 leading-relaxed bg-zinc-950/60 p-4 rounded-2xl border border-zinc-800/80">
        {advisory.clinicalBriefing}
      </p>

      <!-- Actionable Clinical Sleep Protocol -->
      <div class="space-y-2">
        <span class="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
          <CheckCircle2 class="w-4 h-4 text-emerald-400" /> Actionable Recovery Protocol
        </span>
        <div class="space-y-1.5">
          {#each advisory.actionableProtocol as item}
            <div class="flex items-start gap-2 bg-zinc-950/40 p-2.5 rounded-xl border border-zinc-800/50 text-xs text-zinc-300">
              <span class="text-emerald-400 font-bold shrink-0 mt-0.5">•</span>
              <span>{item}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Goblin Sleep Coach & Bedtime Calculator -->
    <div class="lg:col-span-5 bg-gradient-to-b from-zinc-900 via-zinc-900 to-emerald-950/20 border border-emerald-500/30 rounded-3xl p-5 md:p-6 shadow-xl space-y-4 flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
          <div class="flex items-center gap-2">
            <span class="text-xl">👹</span>
            <h3 class="text-base font-bold text-zinc-100">{goblinState.profile?.name || 'Gribble'}'s Sleep Orders</h3>
          </div>
          <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
            Sleep Coach
          </span>
        </div>

        <!-- Goblin Dialogue Speech Box -->
        <div class="my-3 p-4 rounded-2xl bg-zinc-950/90 border border-emerald-500/40 relative text-xs md:text-sm text-zinc-100 leading-relaxed font-medium shadow-md">
          <p>"{advisory.goblinAdvice}"</p>
          <div class="absolute -bottom-2 left-6 w-0 h-0 border-x-[6px] border-x-transparent border-t-[8px] border-t-zinc-950"></div>
        </div>

        <!-- Garmin Overnight Telemetry Metrics -->
        <div class="bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800 space-y-3 mt-4">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-zinc-300 flex items-center gap-1.5">
              <Activity class="w-4 h-4 text-indigo-400" /> Synced Garmin Sleep Metrics
            </span>
            <span class="text-[10px] font-mono font-bold text-emerald-400">Garmin Connect</span>
          </div>

          <div class="grid grid-cols-2 gap-2 text-center">
            <div class="bg-zinc-900 p-2 rounded-xl border border-zinc-800">
              <span class="text-[9px] uppercase text-zinc-500 font-bold block">Respiration</span>
              <span class="text-sm font-black font-mono text-cyan-300">{activeRecord?.respirationRate ?? 14} brpm</span>
            </div>
            <div class="bg-zinc-900 p-2 rounded-xl border border-zinc-800">
              <span class="text-[9px] uppercase text-zinc-500 font-bold block">Pulse Ox (SpO2)</span>
              <span class="text-sm font-black font-mono text-emerald-300">{activeRecord?.spo2 ?? 98}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Pre-Sleep Wind-Down Quick Triggers -->
      <div class="space-y-2 pt-2 border-t border-zinc-800/80">
        <span class="text-[11px] font-bold text-zinc-400 uppercase tracking-wider block">Pre-Sleep Wind-Down Practices</span>
        <div class="grid grid-cols-2 gap-2">
          <a
            href="/yoga"
            class="flex items-center justify-center gap-1.5 p-2 rounded-xl bg-purple-500/15 hover:bg-purple-500/25 border border-purple-500/30 text-purple-300 text-xs font-bold transition-all"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>Yoga Nidra</span>
          </a>

          <a
            href="/meditation"
            class="flex items-center justify-center gap-1.5 p-2 rounded-xl bg-cyan-500/15 hover:bg-cyan-500/25 border border-cyan-500/30 text-cyan-300 text-xs font-bold transition-all"
          >
            <Moon class="w-3.5 h-3.5" />
            <span>Body Scan</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
