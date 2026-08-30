<script lang="ts">
  interface Props {
    isActive: boolean;
    phase: 'Inhale' | 'Hold Full' | 'Exhale' | 'Hold Empty';
    secondsRemainingInPhase: number;
    phaseProgress: number; // 0 to 100%
  }

  let {
    isActive = false,
    phase = 'Inhale',
    secondsRemainingInPhase = 4,
    phaseProgress = 0
  }: Props = $props();

  const phaseColor: Record<string, { ring: string; glow: string; text: string; bg: string }> = {
    'Inhale': { ring: '#06b6d4', glow: 'rgba(6, 182, 212, 0.4)', text: 'text-cyan-300', bg: 'from-cyan-500/20' },
    'Hold Full': { ring: '#10b981', glow: 'rgba(16, 185, 129, 0.4)', text: 'text-emerald-300', bg: 'from-emerald-500/20' },
    'Exhale': { ring: '#a855f7', glow: 'rgba(168, 85, 247, 0.4)', text: 'text-purple-300', bg: 'from-purple-500/20' },
    'Hold Empty': { ring: '#f59e0b', glow: 'rgba(245, 158, 11, 0.4)', text: 'text-amber-300', bg: 'from-amber-500/20' },
  };

  let theme = $derived(phaseColor[phase] || phaseColor.Inhale);
</script>

<div class="relative w-64 h-64 md:w-72 md:h-72 flex items-center justify-center select-none">
  <!-- Dynamic Glow Background -->
  <div 
    class="absolute inset-4 rounded-3xl blur-2xl transition-all duration-1000 opacity-40"
    style="background: {theme.glow};"
  ></div>

  <!-- SVG Geometric Tactical Box -->
  <svg viewBox="0 0 200 200" class="w-full h-full drop-shadow-xl">
    <defs>
      <linearGradient id="boxGrad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color={theme.ring} stop-opacity="0.9" />
        <stop offset="100%" stop-color="#3f3f46" stop-opacity="0.3" />
      </linearGradient>
    </defs>

    <!-- Outer Static Perimeter -->
    <rect
      x="25"
      y="25"
      width="150"
      height="150"
      rx="24"
      fill="none"
      stroke="#27272a"
      stroke-width="3"
      stroke-dasharray="6 6"
    />

    <!-- Animated Breathing Expanding Box -->
    <rect
      x={phase === 'Inhale' || phase === 'Hold Full' ? 30 : 60}
      y={phase === 'Inhale' || phase === 'Hold Full' ? 30 : 60}
      width={phase === 'Inhale' || phase === 'Hold Full' ? 140 : 80}
      height={phase === 'Inhale' || phase === 'Hold Full' ? 140 : 80}
      rx="20"
      fill="none"
      stroke={theme.ring}
      stroke-width="5"
      class="transition-all duration-1000 ease-in-out"
      style="filter: drop-shadow(0 0 8px {theme.glow});"
    />

    <!-- Side Indicator Dots (4 corners of box breathing) -->
    <!-- Top-Left: Inhale -->
    <circle cx="25" cy="25" r="5" fill={phase === 'Inhale' ? theme.ring : '#52525b'} />
    <!-- Top-Right: Hold Full -->
    <circle cx="175" cy="25" r="5" fill={phase === 'Hold Full' ? theme.ring : '#52525b'} />
    <!-- Bottom-Right: Exhale -->
    <circle cx="175" cy="175" r="5" fill={phase === 'Exhale' ? theme.ring : '#52525b'} />
    <!-- Bottom-Left: Hold Empty -->
    <circle cx="25" cy="175" r="5" fill={phase === 'Hold Empty' ? theme.ring : '#52525b'} />
  </svg>

  <!-- Center Status & Second Counter -->
  <div class="absolute inset-0 flex flex-col items-center justify-center text-center pointer-events-none">
    <span class="text-[11px] font-mono uppercase tracking-widest text-zinc-400 font-extrabold mb-0.5">
      4-4-4-4
    </span>
    <span class="text-4xl md:text-5xl font-mono font-black {theme.text} tracking-tight">
      {secondsRemainingInPhase}s
    </span>
    <span class="text-sm font-black text-zinc-100 uppercase tracking-wider mt-1 px-3 py-0.5 rounded-full bg-zinc-900/90 border border-zinc-700/80 shadow-md">
      {phase}
    </span>
  </div>
</div>
