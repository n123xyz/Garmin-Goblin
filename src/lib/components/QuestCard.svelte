<script lang="ts">
  import type { GoblinQuest } from '$lib/types/garmin';
  import { claimQuest } from '$lib/state/goblin.svelte';
  import { CheckCircle2, Gift, Footprints, Moon, Dumbbell, HeartPulse, Flame } from 'lucide-svelte';

  let { quest }: { quest: GoblinQuest } = $props();

  let progress = $derived(
    Math.min(Math.round((quest.current_value / quest.target_value) * 100), 100)
  );

  function getCategoryIcon(cat: string) {
    switch (cat.toLowerCase()) {
      case 'steps':
        return Footprints;
      case 'sleep':
        return Moon;
      case 'workout':
        return Dumbbell;
      case 'recovery':
        return HeartPulse;
      default:
        return Flame;
    }
  }

  let CategoryIcon = $derived(getCategoryIcon(quest.quest_category));
</script>

<div class="bg-zinc-900/90 border {quest.is_completed && !quest.is_claimed ? 'border-amber-500/60 shadow-[0_0_15px_rgba(245,158,11,0.2)]' : 'border-zinc-800'} rounded-2xl p-4 transition-all duration-200 hover:border-zinc-700">
  <div class="flex items-start justify-between gap-3">
    <div class="flex items-start gap-3">
      <div class="w-10 h-10 rounded-xl {quest.is_completed ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40' : 'bg-zinc-800 text-zinc-400'} flex items-center justify-center shrink-0 mt-0.5">
        <CategoryIcon class="w-5 h-5" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-xs px-2 py-0.5 rounded-full font-bold uppercase tracking-wider {quest.is_completed ? 'bg-amber-500/20 text-amber-300' : 'bg-zinc-800 text-zinc-400'}">
            {quest.quest_category}
          </span>
          {#if quest.is_claimed}
            <span class="text-[11px] text-zinc-500 font-semibold flex items-center gap-1">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" /> Claimed
            </span>
          {/if}
        </div>
        <h4 class="text-sm font-bold text-zinc-100 mt-1">{quest.title}</h4>
        <p class="text-xs text-zinc-400 mt-0.5 leading-relaxed">{quest.description}</p>
      </div>
    </div>

    <!-- Rewards Badge -->
    <div class="flex flex-col items-end shrink-0">
      <div class="flex items-center gap-1.5 text-xs font-bold text-amber-400 bg-amber-500/10 px-2.5 py-1 rounded-lg border border-amber-500/20">
        <span>+{quest.reward_xp} XP</span>
        <span>•</span>
        <span>🪙 {quest.reward_gold}</span>
      </div>
    </div>
  </div>

  <!-- Progress Bar & Claim Button -->
  <div class="mt-3.5 pt-3 border-t border-zinc-800/80 flex items-center justify-between gap-4">
    <div class="flex-1">
      <div class="flex items-center justify-between text-[11px] font-semibold text-zinc-400 mb-1">
        <span>Progress</span>
        <span class="font-mono">{Math.round(quest.current_value)} / {quest.target_value} ({progress}%)</span>
      </div>
      <div class="w-full bg-zinc-950 h-2 rounded-full overflow-hidden border border-zinc-800">
        <div 
          class="h-full rounded-full transition-all duration-500 {quest.is_completed ? 'bg-amber-400' : 'bg-emerald-500'}"
          style="width: {progress}%;"
        ></div>
      </div>
    </div>

    {#if quest.is_completed && !quest.is_claimed}
      <button
        onclick={() => claimQuest(quest.id)}
        class="shrink-0 flex items-center gap-1.5 px-4 py-2 rounded-xl bg-amber-500 hover:bg-amber-400 text-zinc-950 font-black text-xs shadow-[0_0_12px_rgba(245,158,11,0.4)] transition-all cursor-pointer animate-pulse"
      >
        <Gift class="w-3.5 h-3.5" />
        <span>Claim</span>
      </button>
    {/if}
  </div>
</div>
