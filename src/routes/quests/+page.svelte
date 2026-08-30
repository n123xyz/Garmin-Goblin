<script lang="ts">
  import { onMount } from 'svelte';
  import { goblinState, loadGoblinProfile, loadQuests } from '$lib/state/goblin.svelte';
  import QuestCard from '$lib/components/QuestCard.svelte';
  import { ScrollText, Trophy, Coins, Crown, Shield, Sparkles, CheckCircle2 } from 'lucide-svelte';

  let goblin = $derived(goblinState.profile);
  let quests = $derived(goblinState.quests);

  let activeQuests = $derived(quests.filter(q => !q.is_claimed));
  let claimedQuests = $derived(quests.filter(q => q.is_claimed));

  onMount(async () => {
    await loadGoblinProfile();
    await loadQuests();
  });

  const evolutionRanks = [
    { rank: 'Cave Scamp', levels: 'Lv 1-3', desc: 'Basic dungeon scavanger powered by casual walks', icon: '🐾' },
    { rank: 'Cavern Scout', levels: 'Lv 4-7', desc: 'Agile runner capable of 10K dungeon tracks', icon: '🏹' },
    { rank: 'Dungeon Raider', levels: 'Lv 8-12', desc: 'Hardened warrior fortified by consistent 80+ sleep scores', icon: '⚔️' },
    { rank: 'Goblin Shaman', levels: 'Lv 13-18', desc: 'Master of autonomic recovery, HRV balance, and breathwork', icon: '🔮' },
    { rank: 'Mountain Behemoth', levels: 'Lv 19-25', desc: 'Heavy strength titan with VO2 max surpassing mortal limits', icon: '🛡️' },
    { rank: 'Goblin Warlord', levels: 'Lv 26+', desc: 'Supreme master of cardiovascular endurance and iron physiology', icon: '👑' },
  ];
</script>

<div class="max-w-5xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Header Banner -->
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-amber-950/30 border border-amber-500/20 rounded-3xl p-5 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(245,158,11,0.3)]">
        📜
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          Goblin Quest Board & Armory
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">Complete physical biometric challenges to level up your goblin.</p>
      </div>
    </div>

    <!-- Gold & Level Summary -->
    <div class="flex items-center gap-3 bg-zinc-950/80 p-3 rounded-2xl border border-zinc-800 shrink-0">
      <div class="flex items-center gap-2 text-amber-400 font-bold text-sm">
        <Coins class="w-5 h-5" />
        <span class="text-lg font-black font-mono">{goblin?.gold || 0}</span>
        <span class="text-xs text-zinc-400">Gold</span>
      </div>
      <div class="w-px h-6 bg-zinc-800"></div>
      <div class="text-sm font-bold text-emerald-400 font-mono">
        Level {goblin?.level || 1}
      </div>
    </div>
  </div>

  <!-- Active Quests -->
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-bold text-zinc-100 flex items-center gap-2">
        <ScrollText class="w-5 h-5 text-amber-400" />
        Active Quests ({activeQuests.length})
      </h2>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each activeQuests as quest (quest.id)}
        <QuestCard {quest} />
      {/each}
    </div>
  </div>

  <!-- Goblin Evolution Ranks Tier List -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Crown class="w-5 h-5 text-amber-400" />
        <h3 class="text-base font-bold text-zinc-100">Goblin Evolution Progression</h3>
      </div>
      <span class="text-xs px-2.5 py-1 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30">
        Current: {goblin?.evolution_rank || 'Cave Scamp'}
      </span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      {#each evolutionRanks as evo}
        {@const isCurrent = goblin?.evolution_rank === evo.rank}
        <div class="p-4 rounded-2xl border transition-all {isCurrent ? 'bg-amber-950/20 border-amber-500/50 shadow-[0_0_15px_rgba(245,158,11,0.2)]' : 'bg-zinc-950/60 border-zinc-800/60'}">
          <div class="flex items-center justify-between mb-2">
            <span class="text-2xl">{evo.icon}</span>
            <span class="text-xs px-2 py-0.5 rounded-full font-bold {isCurrent ? 'bg-amber-500/20 text-amber-300' : 'bg-zinc-800 text-zinc-400'}">
              {evo.levels}
            </span>
          </div>
          <h4 class="text-sm font-bold text-zinc-200 {isCurrent ? 'text-amber-300' : ''}">{evo.rank}</h4>
          <p class="text-xs text-zinc-400 mt-1 leading-relaxed">{evo.desc}</p>
        </div>
      {/each}
    </div>
  </div>

  <!-- Completed & Claimed Quests History -->
  {#if claimedQuests.length > 0}
    <div class="space-y-3 opacity-75">
      <h3 class="text-sm font-bold text-zinc-400 flex items-center gap-2">
        <CheckCircle2 class="w-4 h-4 text-emerald-500" />
        Completed Quests Archive ({claimedQuests.length})
      </h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each claimedQuests as quest (quest.id)}
          <QuestCard {quest} />
        {/each}
      </div>
    </div>
  {/if}
</div>
