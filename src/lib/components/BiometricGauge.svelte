<script lang="ts">
  let {
    label,
    value,
    max = 100,
    unit = '',
    subtitle = '',
    color = 'emerald',
    icon = '⚡',
  }: {
    label: string;
    value: number;
    max?: number;
    unit?: string;
    subtitle?: string;
    color?: 'emerald' | 'cyan' | 'gold' | 'rose' | 'purple' | 'amber';
    icon?: string;
  } = $props();

  let percentage = $derived(Math.min(Math.max((value / max) * 100, 0), 100));
  let circumference = 2 * Math.PI * 38;
  let strokeDashoffset = $derived(circumference - (percentage / 100) * circumference);

  const colorMap = {
    emerald: {
      stroke: '#10b981',
      bg: 'rgba(16, 185, 129, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(16, 185, 129, 0.5))',
      text: 'text-emerald-400',
    },
    cyan: {
      stroke: '#06b6d4',
      bg: 'rgba(6, 182, 212, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(6, 182, 212, 0.5))',
      text: 'text-cyan-400',
    },
    gold: {
      stroke: '#f59e0b',
      bg: 'rgba(245, 158, 11, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(245, 158, 11, 0.5))',
      text: 'text-amber-400',
    },
    rose: {
      stroke: '#f43f5e',
      bg: 'rgba(244, 63, 94, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(244, 63, 94, 0.5))',
      text: 'text-rose-400',
    },
    purple: {
      stroke: '#a855f7',
      bg: 'rgba(168, 85, 247, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(168, 85, 247, 0.5))',
      text: 'text-purple-400',
    },
    amber: {
      stroke: '#fbbf24',
      bg: 'rgba(251, 191, 36, 0.15)',
      glow: 'drop-shadow(0 0 8px rgba(251, 191, 36, 0.5))',
      text: 'text-yellow-400',
    }
  };

  let activeColor = $derived(colorMap[color] || colorMap.emerald);
</script>

<div class="bg-zinc-900/80 border border-zinc-800/80 rounded-2xl p-3.5 flex items-center gap-3.5 shadow-lg backdrop-blur-md relative overflow-hidden transition-all duration-200 hover:border-zinc-700">
  <!-- Radial Progress Ring -->
  <div class="relative w-16 h-16 shrink-0 flex items-center justify-center">
    <svg class="w-full h-full -rotate-90" viewBox="0 0 90 90">
      <!-- Background track -->
      <circle
        cx="45"
        cy="45"
        r="38"
        fill="transparent"
        stroke="#1e293b"
        stroke-width="7"
      />
      <!-- Progress Bar -->
      <circle
        cx="45"
        cy="45"
        r="38"
        fill="transparent"
        stroke={activeColor.stroke}
        stroke-width="7"
        stroke-linecap="round"
        stroke-dasharray={circumference}
        stroke-dashoffset={strokeDashoffset}
        style="filter: {activeColor.glow}; transition: stroke-dashoffset 0.8s ease-out;"
      />
    </svg>
    <div class="absolute inset-0 flex items-center justify-center text-lg">
      {icon}
    </div>
  </div>

  <!-- Metric Text & Labels -->
  <div class="flex-1 min-w-0">
    <span class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block truncate">{label}</span>
    <div class="flex items-baseline gap-1 mt-0.5">
      <span class="text-xl md:text-2xl font-black text-zinc-100 font-mono">{value}</span>
      {#if unit}
        <span class="text-xs text-zinc-400 font-medium">{unit}</span>
      {/if}
    </div>
    {#if subtitle}
      <span class="text-[11px] font-medium {activeColor.text} block truncate mt-0.5">{subtitle}</span>
    {/if}
  </div>
</div>
