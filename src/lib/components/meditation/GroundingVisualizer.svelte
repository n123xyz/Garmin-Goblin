<script lang="ts">
  interface Props {
    currentStepIndex: number;
    activeCount: number; // 5, 4, 3, 2, 1
  }

  let {
    currentStepIndex = 0,
    activeCount = 5
  }: Props = $props();

  interface GroundingSense {
    count: number;
    icon: string;
    title: string;
    prompt: string;
    items: string[];
    color: string;
  }

  const senses: GroundingSense[] = [
    {
      count: 5,
      icon: '👁️',
      title: '5 Things You See',
      prompt: 'Notice small details: shadows, textures, wall cracks, light patterns.',
      items: ['Detail 1', 'Detail 2', 'Detail 3', 'Detail 4', 'Detail 5'],
      color: 'emerald',
    },
    {
      count: 4,
      icon: '✋',
      title: '4 Things You Feel',
      prompt: 'Notice physical touch: chair support, clothes texture, air temperature, feet on floor.',
      items: ['Touch 1', 'Touch 2', 'Touch 3', 'Touch 4'],
      color: 'cyan',
    },
    {
      count: 3,
      icon: '👂',
      title: '3 Things You Hear',
      prompt: 'Listen beyond immediate noise: distant traffic, computer hum, natural breath.',
      items: ['Sound 1', 'Sound 2', 'Sound 3'],
      color: 'amber',
    },
    {
      count: 2,
      icon: '👃',
      title: '2 Things You Smell',
      prompt: 'Detect subtle aromas, or recall favorite scents like rain, coffee, or cedar.',
      items: ['Scent 1', 'Scent 2'],
      color: 'rose',
    },
    {
      count: 1,
      icon: '👅',
      title: '1 Thing You Taste',
      prompt: 'Notice current mouth sensation or the clean neutral taste of water.',
      items: ['Taste 1'],
      color: 'purple',
    },
  ];

  let checkedItems = $state<Record<string, boolean>>({});
  let activeSense = $derived(senses[currentStepIndex] || senses[0]);

  function toggleItem(senseIndex: number, itemIndex: number) {
    const key = `${senseIndex}-${itemIndex}`;
    checkedItems[key] = !checkedItems[key];
  }
</script>

<div class="w-full space-y-4">
  <!-- Sensory Count Cards -->
  <div class="grid grid-cols-1 md:grid-cols-5 gap-2.5">
    {#each senses as sense, sIdx}
      {@const isCurrent = currentStepIndex === sIdx}
      {@const isPast = currentStepIndex > sIdx}
      <div 
        class="rounded-2xl p-3 border transition-all flex flex-col justify-between {isCurrent ? 'bg-emerald-500/15 border-emerald-500/60 shadow-lg shadow-emerald-500/10' : isPast ? 'bg-zinc-950/90 border-zinc-800 text-zinc-400' : 'bg-zinc-950/40 border-zinc-800/60 opacity-60'}"
      >
        <div class="flex items-center justify-between">
          <span class="text-xl">{sense.icon}</span>
          <span class="text-xs font-mono font-black px-2 py-0.5 rounded-full {isCurrent ? 'bg-emerald-500/20 text-emerald-300' : 'bg-zinc-800 text-zinc-400'}">
            {sense.count}
          </span>
        </div>
        <div class="mt-2">
          <h4 class="text-xs font-bold {isCurrent ? 'text-zinc-100' : 'text-zinc-400'}">
            {sense.title}
          </h4>
        </div>
      </div>
    {/each}
  </div>

  <!-- Active Sense Detail Card & Interactive Tap Checkpoints -->
  <div class="bg-zinc-950/80 border border-zinc-800 rounded-2xl p-4 md:p-5 space-y-3">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-2">
      <div class="flex items-center gap-2">
        <span class="text-2xl">{activeSense.icon}</span>
        <div>
          <h3 class="text-sm md:text-base font-black text-zinc-100">{activeSense.title}</h3>
          <p class="text-xs text-zinc-400">{activeSense.prompt}</p>
        </div>
      </div>
      <span class="text-xs font-mono text-emerald-400 font-bold">
        {Object.keys(checkedItems).filter(k => k.startsWith(`${currentStepIndex}-`) && checkedItems[k]).length} / {activeSense.count} Noted
      </span>
    </div>

    <div class="flex flex-wrap gap-2 pt-1">
      {#each activeSense.items as item, iIdx}
        {@const isChecked = !!checkedItems[`${currentStepIndex}-${iIdx}`]}
        <button
          onclick={() => toggleItem(currentStepIndex, iIdx)}
          class="px-3.5 py-2 rounded-xl border text-xs font-bold transition-all cursor-pointer flex items-center gap-2 {isChecked ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50 shadow-md shadow-emerald-500/10' : 'bg-zinc-900/90 text-zinc-300 border-zinc-800 hover:border-zinc-700'}"
        >
          <span>{isChecked ? '✓' : '○'}</span>
          <span>{item}</span>
        </button>
      {/each}
    </div>
  </div>
</div>
