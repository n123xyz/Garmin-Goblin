<script lang="ts">
  interface Props {
    targetZone?: string; // 'full-body' | 'right-leg' | 'left-leg' | 'torso' | 'arms' | 'head'
    zoneTitle?: string;
  }

  let {
    targetZone = 'full-body',
    zoneTitle = 'Full Body Awareness'
  }: Props = $props();

  const isZone = (zone: string) => targetZone === zone || targetZone === 'full-body';
</script>

<div class="relative w-64 h-64 md:w-72 md:h-72 flex items-center justify-center select-none">
  <!-- Glowing Ambient Aura -->
  <div 
    class="absolute inset-4 rounded-full blur-2xl transition-all duration-1000 opacity-30 bg-purple-500/40"
  ></div>

  <!-- Anatomical Vector Silhouette -->
  <svg viewBox="0 0 200 240" class="w-full h-full drop-shadow-xl" xmlns="http://www.w3.org/2000/svg">
    <defs>
      <!-- Zone Glow Gradients -->
      <radialGradient id="nodeGlow" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#c084fc" stop-opacity="1" />
        <stop offset="100%" stop-color="#9333ea" stop-opacity="0" />
      </radialGradient>
    </defs>

    <!-- Base Anatomical Silhouette (Subtle Outline) -->
    <g stroke="#3f3f46" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round">
      <!-- Head & Neck -->
      <circle cx="100" cy="30" r="14" fill="#18181b" stroke={isZone('head') ? '#c084fc' : '#3f3f46'} stroke-width={isZone('head') ? '3.5' : '2.5'} />
      <line x1="100" y1="44" x2="100" y2="52" stroke={isZone('head') ? '#c084fc' : '#3f3f46'} />

      <!-- Torso & Pelvis -->
      <path 
        d="M 80 54 L 120 54 L 115 120 L 85 120 Z" 
        fill={isZone('torso') ? 'rgba(192, 132, 252, 0.15)' : '#18181b'} 
        stroke={isZone('torso') ? '#c084fc' : '#3f3f46'} 
        stroke-width={isZone('torso') ? '3.5' : '2.5'} 
      />

      <!-- Left Arm -->
      <path 
        d="M 80 54 L 56 100 L 48 140" 
        stroke={isZone('arms') ? '#c084fc' : '#3f3f46'} 
        stroke-width={isZone('arms') ? '3.5' : '2.5'} 
      />
      <circle cx="46" cy="144" r="5" fill={isZone('arms') ? '#c084fc' : '#27272a'} />

      <!-- Right Arm -->
      <path 
        d="M 120 54 L 144 100 L 152 140" 
        stroke={isZone('arms') ? '#c084fc' : '#3f3f46'} 
        stroke-width={isZone('arms') ? '3.5' : '2.5'} 
      />
      <circle cx="154" cy="144" r="5" fill={isZone('arms') ? '#c084fc' : '#27272a'} />

      <!-- Left Leg -->
      <path 
        d="M 90 120 L 84 170 L 82 220" 
        stroke={isZone('left-leg') ? '#c084fc' : '#3f3f46'} 
        stroke-width={isZone('left-leg') ? '3.5' : '2.5'} 
      />
      <circle cx="82" cy="225" r="5" fill={isZone('left-leg') ? '#c084fc' : '#27272a'} />

      <!-- Right Leg -->
      <path 
        d="M 110 120 L 116 170 L 118 220" 
        stroke={isZone('right-leg') ? '#c084fc' : '#3f3f46'} 
        stroke-width={isZone('right-leg') ? '3.5' : '2.5'} 
      />
      <circle cx="118" cy="225" r="5" fill={isZone('right-leg') ? '#c084fc' : '#27272a'} />
    </g>

    <!-- Active Energy Scan Points -->
    {#if isZone('head')}
      <circle cx="100" cy="30" r="18" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
      <circle cx="100" cy="30" r="4" fill="#ffffff" />
    {/if}

    {#if isZone('torso')}
      <circle cx="100" cy="85" r="24" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
      <circle cx="100" cy="85" r="4" fill="#ffffff" />
    {/if}

    {#if isZone('arms')}
      <circle cx="50" cy="120" r="14" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
      <circle cx="150" cy="120" r="14" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
    {/if}

    {#if isZone('right-leg')}
      <circle cx="118" cy="170" r="18" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
      <circle cx="118" cy="225" r="8" fill="url(#nodeGlow)" opacity="0.8" />
    {/if}

    {#if isZone('left-leg')}
      <circle cx="84" cy="170" r="18" fill="url(#nodeGlow)" opacity="0.6" class="animate-pulse" />
      <circle cx="82" cy="225" r="8" fill="url(#nodeGlow)" opacity="0.8" />
    {/if}

    {#if targetZone === 'full-body'}
      <circle cx="100" cy="120" r="45" stroke="#c084fc" stroke-width="1.5" stroke-dasharray="4 4" opacity="0.5" class="animate-ping" />
    {/if}
  </svg>

  <!-- Bottom Target Zone Pill -->
  <div class="absolute bottom-2 inset-x-0 flex justify-center pointer-events-none">
    <span class="text-[11px] font-mono font-bold px-3 py-1 rounded-full bg-zinc-950/90 text-purple-300 border border-purple-500/40 shadow-lg">
      Focus: {zoneTitle}
    </span>
  </div>
</div>
