<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Wind, Send, Plus } from 'lucide-svelte';

  interface FloatingLeaf {
    id: number;
    text: string;
    x: number; // 0 to 100%
    y: number; // vertical lane (20 to 80%)
    rotation: number;
    scale: number;
    speed: number;
    color: string;
  }

  let leaves = $state<FloatingLeaf[]>([]);
  let customThought = $state('');
  let animFrame: number | null = null;
  let autoLeafTimer: ReturnType<typeof setInterval> | null = null;
  let nextId = 1;

  const leafColors = ['#10b981', '#059669', '#34d399', '#f59e0b', '#d97706', '#84cc16'];

  onMount(() => {
    // Spawn a couple initial gentle leaves
    spawnLeaf('Gentle morning breath');
    spawnLeaf('Letting go of tension');

    // Auto-spawn ambient leaves every 6 seconds
    autoLeafTimer = setInterval(() => {
      if (leaves.length < 5) {
        const thoughts = ['Just a passing thought', 'Release and soften', 'Watching without clinging', 'Inhale calm', 'Exhale ease'];
        const randomThought = thoughts[Math.floor(Math.random() * thoughts.length)];
        spawnLeaf(randomThought);
      }
    }, 6000);

    // Animation Loop for stream drift
    function tick() {
      leaves = leaves
        .map((leaf) => ({
          ...leaf,
          x: leaf.x + leaf.speed,
          rotation: leaf.rotation + 0.15,
        }))
        .filter((leaf) => leaf.x < 115); // Discard leaves that floated off right bank

      animFrame = requestAnimationFrame(tick);
    }
    animFrame = requestAnimationFrame(tick);
  });

  onDestroy(() => {
    if (animFrame) cancelAnimationFrame(animFrame);
    if (autoLeafTimer) clearInterval(autoLeafTimer);
  });

  function spawnLeaf(thoughtText?: string) {
    const text = (thoughtText || customThought || 'Mind in stillness').trim();
    const newLeaf: FloatingLeaf = {
      id: nextId++,
      text,
      x: -15,
      y: 20 + Math.random() * 55, // Random water lane
      rotation: Math.random() * 360,
      scale: 0.85 + Math.random() * 0.3,
      speed: 0.08 + Math.random() * 0.06,
      color: leafColors[Math.floor(Math.random() * leafColors.length)],
    };
    leaves = [...leaves, newLeaf];
    customThought = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      spawnLeaf();
    }
  }
</script>

<div class="w-full space-y-4 select-none">
  <!-- River Visual Stream Canvas -->
  <div class="relative w-full h-64 md:h-72 rounded-3xl overflow-hidden border border-emerald-500/30 bg-gradient-to-b from-[#0b1f1a] via-[#091815] to-[#071311] shadow-2xl">
    <!-- Top & Bottom Riverbank Shimmer -->
    <div class="absolute inset-x-0 top-0 h-4 bg-gradient-to-b from-emerald-950/80 to-transparent pointer-events-none"></div>
    <div class="absolute inset-x-0 bottom-0 h-4 bg-gradient-to-t from-emerald-950/80 to-transparent pointer-events-none"></div>

    <!-- Animated Water Ripples Background -->
    <svg class="absolute inset-0 w-full h-full opacity-20 pointer-events-none" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <pattern id="riverPattern" width="120" height="40" patternUnits="userSpaceOnUse">
          <path d="M 0 20 Q 30 10, 60 20 T 120 20" fill="none" stroke="#34d399" stroke-width="1.5" />
          <path d="M 0 35 Q 30 25, 60 35 T 120 35" fill="none" stroke="#06b6d4" stroke-width="1" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" fill="url(#riverPattern)" />
    </svg>

    <!-- Floating Thought Leaves on Water -->
    {#each leaves as leaf (leaf.id)}
      <div
        class="absolute transition-transform will-change-transform flex items-center gap-2 pointer-events-none"
        style="left: {leaf.x}%; top: {leaf.y}%; transform: scale({leaf.scale});"
      >
        <!-- Leaf Vector Icon -->
        <div 
          class="w-10 h-10 rounded-full flex items-center justify-center shadow-lg transition-transform"
          style="transform: rotate({leaf.rotation}deg); filter: drop-shadow(0 0 8px {leaf.color}50);"
        >
          <svg viewBox="0 0 100 100" class="w-8 h-8">
            <path
              d="M 50 10 C 20 25 15 65 50 90 C 85 65 80 25 50 10 Z"
              fill={leaf.color}
              opacity="0.9"
            />
            <path
              d="M 50 10 L 50 90 M 50 30 Q 35 40 25 50 M 50 50 Q 65 60 75 70"
              stroke="#064e3b"
              stroke-width="3"
              fill="none"
            />
          </svg>
        </div>

        <!-- Attached Thought Badge -->
        <span class="text-xs font-medium px-3 py-1 rounded-xl bg-zinc-950/90 text-emerald-200 border border-emerald-500/40 shadow-xl max-w-[200px] truncate backdrop-blur-sm">
          {leaf.text}
        </span>
      </div>
    {/each}

    <!-- Water Stream Ambient Flow Hint -->
    <div class="absolute bottom-3 right-4 flex items-center gap-1.5 text-[10px] font-mono text-emerald-400/80 bg-zinc-950/70 px-2.5 py-1 rounded-lg border border-emerald-500/30">
      <Wind class="w-3 h-3 animate-spin" />
      <span>Observing without judgment</span>
    </div>
  </div>

  <!-- Interactive Thought Placement Bar -->
  <div class="flex gap-2">
    <input
      type="text"
      bind:value={customThought}
      onkeydown={handleKeydown}
      placeholder="Type a thought to place on a leaf (or leave blank to spawn)..."
      class="flex-1 bg-zinc-950 border border-zinc-800 focus:border-amber-500 rounded-2xl px-4 py-2.5 text-xs md:text-sm text-zinc-100 placeholder:text-zinc-600 focus:outline-none transition-colors"
    />

    <button
      onclick={() => spawnLeaf()}
      class="px-4 py-2.5 rounded-2xl bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-zinc-950 font-black text-xs md:text-sm flex items-center gap-1.5 transition-all shadow-lg shadow-amber-500/20 cursor-pointer shrink-0"
    >
      <span>Place on Leaf</span>
      <span class="text-base">🍃</span>
    </button>
  </div>
</div>
