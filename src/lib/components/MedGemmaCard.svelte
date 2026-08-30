<script lang="ts">
  import type { HealthInsight } from '$lib/types/garmin';
  import { generateMedGemmaInsight, biometricsState } from '$lib/state/biometrics.svelte';
  import { Sparkles, Activity, ShieldCheck, HeartPulse, RefreshCw } from 'lucide-svelte';

  let { insight }: { insight?: HealthInsight } = $props();

  let activeInsight = $derived(insight || biometricsState.insights[0]);
  let isAnalyzing = $derived(biometricsState.isAnalyzing);
</script>

<div class="bg-gradient-to-br from-zinc-900 via-zinc-900 to-emerald-950/40 border border-emerald-500/30 rounded-3xl p-5 md:p-6 shadow-2xl relative overflow-hidden">
  <!-- Glowing Background Accent -->
  <div class="absolute -right-16 -top-16 w-48 h-48 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none"></div>

  <!-- Header -->
  <div class="flex items-center justify-between border-b border-zinc-800/80 pb-4 mb-4">
    <div class="flex items-center gap-2.5">
      <div class="w-9 h-9 rounded-xl bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400">
        <HeartPulse class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-base md:text-lg font-bold text-zinc-100 flex items-center gap-2">
          MedGemma Biometric Intelligence
          <span class="text-[10px] px-2 py-0.5 rounded-full font-extrabold bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 uppercase tracking-wider">
            Clinical AI
          </span>
        </h3>
        <span class="text-xs text-zinc-400">On-Device Autonomous Health Reasoning</span>
      </div>
    </div>

    <button
      onclick={generateMedGemmaInsight}
      disabled={isAnalyzing}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/30 text-emerald-300 text-xs font-semibold transition-all disabled:opacity-50 cursor-pointer"
    >
      <RefreshCw class="w-3.5 h-3.5 {isAnalyzing ? 'animate-spin' : ''}" />
      <span>{isAnalyzing ? 'Analyzing...' : 'Re-Evaluate'}</span>
    </button>
  </div>

  {#if activeInsight}
    <!-- Readiness Score Banner -->
    <div class="bg-zinc-950/60 rounded-2xl p-4 border border-zinc-800 flex items-center justify-between mb-4">
      <div>
        <span class="text-xs text-zinc-400 font-semibold uppercase tracking-wider block">Physiological Readiness</span>
        <span class="text-xl md:text-2xl font-black text-emerald-400 tracking-tight font-mono">{activeInsight.readiness_state}</span>
      </div>
      <div class="text-right">
        <span class="text-3xl font-black text-zinc-100 font-mono">{activeInsight.readiness_score}</span>
        <span class="text-xs text-zinc-500 block">/ 100 PTS</span>
      </div>
    </div>

    <!-- Clinical Assessment -->
    <div class="space-y-3">
      <div>
        <span class="text-xs font-bold uppercase tracking-wider text-emerald-400/90 flex items-center gap-1.5 mb-1">
          <Activity class="w-3.5 h-3.5" />
          Clinical Autonomic Evaluation
        </span>
        <p class="text-xs md:text-sm text-zinc-300 leading-relaxed bg-zinc-900/60 p-3 rounded-xl border border-zinc-800/60">
          {activeInsight.clinical_summary}
        </p>
      </div>

      <!-- Goblin Tough-Love Reaction -->
      <div>
        <span class="text-xs font-bold uppercase tracking-wider text-amber-400 flex items-center gap-1.5 mb-1">
          <Sparkles class="w-3.5 h-3.5" />
          Goblin Coach Translation
        </span>
        <p class="text-xs md:text-sm text-amber-100/90 leading-relaxed bg-amber-950/20 border border-amber-500/30 p-3 rounded-xl italic font-medium">
          "{activeInsight.goblin_reaction}"
        </p>
      </div>

      <!-- Actionable Directives -->
      {#if activeInsight.actionable_tips}
        <div class="pt-1">
          <span class="text-xs font-bold uppercase tracking-wider text-cyan-400 flex items-center gap-1.5 mb-1">
            <ShieldCheck class="w-3.5 h-3.5" />
            Prescribed Protocols
          </span>
          <div class="text-xs text-zinc-300 whitespace-pre-line bg-zinc-950/40 p-3 rounded-xl border border-zinc-800/50 leading-relaxed font-mono">
            {activeInsight.actionable_tips}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="text-center py-8 text-zinc-500">
      <p class="text-sm">Click "Re-Evaluate" to generate your first MedGemma Biometric Insight.</p>
    </div>
  {/if}
</div>
