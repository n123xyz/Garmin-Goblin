<script lang="ts">
  interface Props {
    poseId: string;
    class?: string;
    isActive?: boolean;
    accentColor?: string;
  }

  let {
    poseId,
    class: className = 'w-32 h-32',
    isActive = false,
    accentColor = 'emerald'
  }: Props = $props();

  const colorMap: Record<string, { stroke: string; glow: string; fill: string }> = {
    emerald: { stroke: '#10b981', glow: 'rgba(16, 185, 129, 0.4)', fill: '#059669' },
    cyan: { stroke: '#06b6d4', glow: 'rgba(6, 182, 212, 0.4)', fill: '#0891b2' },
    amber: { stroke: '#f59e0b', glow: 'rgba(245, 158, 11, 0.4)', fill: '#d97706' },
    rose: { stroke: '#f43f5e', glow: 'rgba(244, 63, 94, 0.4)', fill: '#e11d48' },
    purple: { stroke: '#a855f7', glow: 'rgba(168, 85, 247, 0.4)', fill: '#9333ea' },
  };

  let theme = $derived(colorMap[accentColor] || colorMap.emerald);
</script>

<div class="relative flex items-center justify-center {className}">
  {#if isActive}
    <div 
      class="absolute inset-0 rounded-full blur-xl opacity-30 animate-pulse transition-opacity duration-1000"
      style="background: radial-gradient(circle, {theme.glow} 0%, transparent 70%);"
    ></div>
  {/if}

  <svg
    viewBox="0 0 200 200"
    xmlns="http://www.w3.org/2000/svg"
    class="w-full h-full drop-shadow-md select-none transition-transform duration-500 {isActive ? 'scale-105' : ''}"
  >
    <defs>
      <!-- Mat Ground Gradient -->
      <linearGradient id="groundGrad-{poseId}" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stop-color="#3f3f46" stop-opacity="0.2" />
        <stop offset="25%" stop-color="#71717a" stop-opacity="0.8" />
        <stop offset="50%" stop-color={theme.stroke} stop-opacity="0.9" />
        <stop offset="75%" stop-color="#71717a" stop-opacity="0.8" />
        <stop offset="100%" stop-color="#3f3f46" stop-opacity="0.2" />
      </linearGradient>

      <!-- Energy Flow Gradient -->
      <linearGradient id="flowGrad-{poseId}" x1="0%" y1="100%" x2="100%" y2="0%">
        <stop offset="0%" stop-color="#71717a" />
        <stop offset="100%" stop-color={theme.stroke} />
      </linearGradient>
    </defs>

    <!-- Ground Yoga Mat Anchor -->
    <line
      x1="20"
      y1="172"
      x2="180"
      y2="172"
      stroke="url(#groundGrad-{poseId})"
      stroke-width="3.5"
      stroke-linecap="round"
    />

    <!-- Pose Geometry Group -->
    <g
      stroke={isActive ? theme.stroke : 'currentColor'}
      stroke-width="5"
      stroke-linecap="round"
      stroke-linejoin="round"
      fill="none"
      class="transition-colors duration-300"
    >
      {#if poseId === 'childs-pose'}
        <!-- Child's Pose (Balasana) -->
        <!-- Folded Legs / Hips -->
        <path d="M 38 168 C 45 138 72 138 82 152 C 86 158 84 168 50 168 Z" stroke-width="4.5" fill="rgba(113, 113, 122, 0.15)" />
        <!-- Lowered Spine / Torso -->
        <path d="M 78 146 C 105 138 132 144 150 156" stroke-width="5.5" />
        <!-- Arms Extended Forward -->
        <path d="M 125 146 C 145 152 165 162 178 168" stroke-width="4.5" />
        <!-- Head Resting on Mat -->
        <circle cx="156" cy="158" r="10" fill={isActive ? theme.stroke : 'currentColor'} />

      {:else if poseId === 'downward-dog'}
        <!-- Downward-Facing Dog (Adho Mukha Svanasana) -->
        <!-- Left Leg (back) -->
        <line x1="100" y1="52" x2="42" y2="168" stroke-width="5.5" />
        <!-- Spine to Pelvis Apex -->
        <line x1="100" y1="52" x2="140" y2="108" stroke-width="5.5" />
        <!-- Arms to Ground -->
        <line x1="140" y1="108" x2="168" y2="168" stroke-width="5.5" />
        <!-- Head Inverted -->
        <circle cx="138" cy="120" r="10" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Alignment Energy Flow Hint -->
        {#if isActive}
          <path d="M 50 155 Q 100 65 160 155" stroke={theme.stroke} stroke-width="1.5" stroke-dasharray="4 4" opacity="0.6" />
        {/if}

      {:else if poseId === 'warrior-2'}
        <!-- Warrior II (Virabhadrasana II) -->
        <!-- Head & Upright Neck -->
        <circle cx="102" cy="40" r="10.5" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Spine / Torso -->
        <line x1="102" y1="52" x2="102" y2="110" stroke-width="5.5" />
        <!-- Extended Horizontal Arms -->
        <line x1="32" y1="72" x2="172" y2="72" stroke-width="4.5" />
        <!-- Back Straight Leg -->
        <line x1="102" y1="110" x2="42" y2="168" stroke-width="5.5" />
        <!-- Front Bent Lunge Leg (90 deg over ankle) -->
        <polyline points="102,110 155,110 155,168" stroke-width="5.5" />

      {:else if poseId === 'tree-pose'}
        <!-- Tree Pose (Vrksasana) -->
        <!-- Head -->
        <circle cx="100" cy="34" r="10" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Upright Lengthened Spine -->
        <line x1="100" y1="46" x2="100" y2="112" stroke-width="5.5" />
        <!-- Standing Rooted Leg -->
        <line x1="100" y1="112" x2="100" y2="168" stroke-width="5.5" />
        <!-- Bent Leg Folded onto Thigh -->
        <polyline points="100,112 144,128 106,138" stroke-width="4.5" />
        <!-- Anjali Mudra Prayer Arms -->
        <path d="M 100 62 L 78 82 L 100 90 L 122 82 Z" stroke-width="3.5" fill="rgba(113, 113, 122, 0.2)" />

      {:else if poseId === 'cobra-pose'}
        <!-- Cobra Pose (Bhujangasana) -->
        <!-- Flat Legs on Mat -->
        <path d="M 28 166 L 90 166" stroke-width="5.5" />
        <!-- Sweeping Arching Spine -->
        <path d="M 90 166 C 120 166 148 145 156 94" stroke-width="5.5" />
        <!-- Lifted Head & Throat -->
        <circle cx="162" cy="74" r="10.5" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Supporting Arms & Elbows -->
        <polyline points="144,118 136,166 148,168" stroke-width="4.5" />

      {:else if poseId === 'seated-forward-fold'}
        <!-- Seated Forward Fold (Paschimottanasana) -->
        <!-- Extended Legs Flat on Ground -->
        <line x1="48" y1="166" x2="168" y2="166" stroke-width="5.5" />
        <!-- Folded Curved Spine -->
        <path d="M 48 166 C 68 114 112 118 142 146" stroke-width="5.5" />
        <!-- Relaxed Folded Head -->
        <circle cx="148" cy="142" r="10" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Arms Reaching for Feet -->
        <path d="M 108 128 L 166 162" stroke-width="4.5" />

      {:else if poseId === 'bridge-pose'}
        <!-- Bridge Pose (Setu Bandhasana) -->
        <!-- Head & Shoulder Base on Ground -->
        <circle cx="34" cy="158" r="10" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Elevated Pelvis & Spine Arch -->
        <path d="M 46 166 C 76 138 106 88 126 92" stroke-width="5.5" />
        <!-- Thighs to Knees -->
        <line x1="126" y1="92" x2="160" y2="94" stroke-width="5.5" />
        <!-- Calves to Planted Feet -->
        <line x1="160" y1="94" x2="166" y2="168" stroke-width="5.5" />
        <!-- Arms Grounded Along Mat -->
        <line x1="46" y1="168" x2="122" y2="168" stroke-width="4.5" />

      {:else if poseId === 'corpse-pose'}
        <!-- Corpse Pose (Savasana) -->
        <!-- Neutral Head on Mat -->
        <circle cx="38" cy="154" r="10" fill={isActive ? theme.stroke : 'currentColor'} />
        <!-- Horizontal Torso -->
        <line x1="50" y1="162" x2="118" y2="162" stroke-width="5.5" />
        <!-- Relaxed Splayed Legs -->
        <line x1="118" y1="162" x2="174" y2="166" stroke-width="4.5" />
        <line x1="118" y1="162" x2="174" y2="158" stroke-width="4.5" />
        <!-- Relaxed Open Arms at Sides -->
        <line x1="72" y1="162" x2="124" y2="154" stroke-width="3.5" />
        <line x1="72" y1="162" x2="124" y2="170" stroke-width="3.5" />
        {#if isActive}
          <!-- Serene Pulse Waves -->
          <circle cx="100" cy="140" r="28" stroke={theme.stroke} stroke-width="1" opacity="0.3" stroke-dasharray="3 3" class="animate-ping" />
        {/if}

      {:else}
        <!-- Generic Yoga Pose Fallback -->
        <circle cx="100" cy="60" r="12" fill={isActive ? theme.stroke : 'currentColor'} />
        <line x1="100" y1="74" x2="100" y2="140" stroke-width="5.5" />
        <line x1="100" y1="140" x2="60" y2="168" stroke-width="5" />
        <line x1="100" y1="140" x2="140" y2="168" stroke-width="5" />
        <line x1="50" y1="100" x2="150" y2="100" stroke-width="5" />
      {/if}
    </g>
  </svg>
</div>
