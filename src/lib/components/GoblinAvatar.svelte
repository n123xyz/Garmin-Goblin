<script lang="ts">
  import { goblinState, setGoblinSpeech } from '$lib/state/goblin.svelte';
  import { biometricsState } from '$lib/state/biometrics.svelte';
  import { formatSecondsToHoursMinutes } from '$lib/state/sleep.svelte';

  let isPoked = $state(false);

  const pokeQuotes = [
    "HEHE! Watch the pointy ears, human! Did you bring a meal to feed Gribble?",
    "Poke me again and I will double your step goal tomorrow!",
    "GRAH! Feed Gribble your meal photo or water intake!",
    "I am powered by your Body Battery & nutrition! Keep it above 50%!",
    "Less poking, more feeding Gribble and dungeon walking!",
  ];

  function handlePoke() {
    isPoked = true;
    const randomQuote = pokeQuotes[Math.floor(Math.random() * pokeQuotes.length)];
    setGoblinSpeech(randomQuote, 4000);
    setTimeout(() => {
      isPoked = false;
    }, 600);
  }

  let mood = $derived(goblinState.profile?.mood || 'Energetic');
  let level = $derived(goblinState.profile?.level || 1);
  let rank = $derived(goblinState.profile?.evolution_rank || 'Cave Scamp');

  // Default dynamic contextual dialogue if no temporary speech is active
  let defaultDialogue = $derived.by(() => {
    const bio = biometricsState.current;
    if (bio && bio.sleep_score > 0 && bio.sleep_score < 65) {
      const actualStr = bio.sleep_duration_sec > 0 ? formatSecondsToHoursMinutes(bio.sleep_duration_sec) : 'briefly';
      return `You logged ${actualStr} of sleep (Score: ${bio.sleep_score}/100). Take it easy in the cavern today!`;
    }
    if (bio && bio.body_battery > 80) {
      return "GRAH! Body Battery is OVER 80%! Let's conquer a dungeon workout!";
    }
    if (bio && bio.steps >= 10000) {
      return "10,000 steps crushed! You are a legendary dungeon crawler!";
    }
    if (mood === 'Zen') {
      return "Inhale tranquility, exhale chaos... My goblin mind is at peace.";
    }
    if (mood === 'Exhausted' || mood === 'Tired') {
      return "YAWN... My goblin energy is drained! Feed Gribble and get some rest in the cave!";
    }
    return "Sync your Garmin watch and feed Gribble with meals and steps, human!";
  });

  let speech = $derived(goblinState.activeSpeechBubble || defaultDialogue);

  // Dynamic eye and color themes based on mood
  let eyeColor = $derived(
    mood === 'Feral' ? '#ef4444' :
    mood === 'Zen' ? '#38bdf8' :
    mood === 'Proud' ? '#fbbf24' :
    mood === 'Exhausted' || mood === 'Tired' ? '#64748b' :
    '#fde047'
  );

  let skinGradient = $derived(
    mood === 'Feral' ? ['#065f46', '#047857', '#ef4444'] :
    mood === 'Exhausted' || mood === 'Tired' ? ['#1e293b', '#334155', '#475569'] :
    mood === 'Zen' ? ['#047857', '#0d9488', '#06b6d4'] :
    mood === 'Proud' ? ['#059669', '#10b981', '#fbbf24'] :
    ['#047857', '#10b981', '#34d399']
  );
</script>

<div class="relative flex flex-col items-center justify-center select-none w-full max-w-sm mx-auto">
  <!-- Speech Bubble (Always clearly positioned and readable above Gribble) -->
  <div class="w-full flex justify-center mb-2 px-1 min-h-[50px] z-20">
    <div 
      class="relative max-w-xs md:max-w-sm px-3.5 py-2 rounded-2xl bg-zinc-950/95 border border-emerald-500/50 text-zinc-100 text-xs md:text-sm font-semibold shadow-[0_4px_20px_rgba(0,0,0,0.7)] backdrop-blur-md transition-all duration-300 text-center flex items-center justify-center gap-1.5"
    >
      <span class="text-emerald-400 shrink-0 text-sm">💬</span>
      <p class="leading-snug text-zinc-100 font-medium">{speech}</p>
      <!-- Speech Tail pointing down to Gribble's head -->
      <div class="absolute -bottom-2 left-1/2 -translate-x-1/2 w-0 h-0 border-x-[6px] border-x-transparent border-t-[8px] border-t-zinc-950"></div>
    </div>
  </div>

  <!-- Goblin Visual Container -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="relative cursor-pointer transition-transform duration-300 active:scale-95 {isPoked ? 'scale-110 rotate-3' : 'hover:scale-105'}"
    onclick={handlePoke}
  >
    <!-- Background Aura Glow -->
    <div class="absolute inset-0 rounded-full blur-3xl opacity-40 {mood === 'Feral' ? 'bg-red-500' : mood === 'Zen' ? 'bg-cyan-500' : 'bg-emerald-500'}"></div>

    <!-- Animated SVG Goblin Pet -->
    <svg 
      viewBox="0 0 240 240" 
      class="w-48 h-48 md:w-56 md:h-56 relative z-10 filter drop-shadow-[0_10px_15px_rgba(0,0,0,0.6)] {mood === 'Sleeping' ? 'opacity-80' : 'animate-goblin-breathe'}"
    >
      <defs>
        <linearGradient id="goblinSkin" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color={skinGradient[0]} />
          <stop offset="60%" stop-color={skinGradient[1]} />
          <stop offset="100%" stop-color={skinGradient[2]} />
        </linearGradient>
        <radialGradient id="eyeGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color={eyeColor} />
          <stop offset="100%" stop-color="#000000" />
        </radialGradient>
      </defs>

      <!-- Goblin Ears (Pointy) -->
      <!-- Left Ear -->
      <polygon points="50,110 5,60 70,85" fill="url(#goblinSkin)" stroke="#064e3b" stroke-width="2" />
      <polygon points="52,105 18,68 66,85" fill="#f43f5e" opacity="0.3" />

      <!-- Right Ear -->
      <polygon points="190,110 235,60 170,85" fill="url(#goblinSkin)" stroke="#064e3b" stroke-width="2" />
      <polygon points="188,105 222,68 174,85" fill="#f43f5e" opacity="0.3" />

      <!-- Goblin Head -->
      <ellipse cx="120" cy="115" rx="65" ry="55" fill="url(#goblinSkin)" stroke="#064e3b" stroke-width="3" />

      <!-- Level Evolution Crown / Headpiece -->
      {#if level >= 8}
        <path d="M 85,75 L 100,50 L 120,65 L 140,50 L 155,75 Z" fill="#fbbf24" stroke="#d97706" stroke-width="2" />
        <circle cx="120" cy="62" r="3" fill="#ef4444" />
      {/if}

      <!-- Cheeks -->
      <circle cx="80" cy="135" r="10" fill="#f43f5e" opacity="0.25" />
      <circle cx="160" cy="135" r="10" fill="#f43f5e" opacity="0.25" />

      <!-- Eyes -->
      {#if mood === 'Sleeping'}
        <!-- Sleeping Eyes -->
        <path d="M 85,115 Q 98,125 110,115" stroke="#e2e8f0" stroke-width="4" fill="none" stroke-linecap="round" />
        <path d="M 130,115 Q 142,125 155,115" stroke="#e2e8f0" stroke-width="4" fill="none" stroke-linecap="round" />
      {:else if mood === 'Zen'}
        <!-- Zen Eyes (Calm slit) -->
        <path d="M 85,115 Q 98,105 110,115" stroke={eyeColor} stroke-width="4" fill="none" stroke-linecap="round" />
        <path d="M 130,115 Q 142,105 155,115" stroke={eyeColor} stroke-width="4" fill="none" stroke-linecap="round" />
        <!-- Third Eye Rune -->
        <circle cx="120" cy="85" r="5" fill="#38bdf8" class="animate-pulse" />
      {:else}
        <!-- Left Eye -->
        <ellipse cx="98" cy="110" rx="14" ry="18" fill="#111827" stroke="#000" stroke-width="1.5" />
        <circle cx="98" cy="110" r="11" fill="url(#eyeGlow)" />
        <circle cx="95" cy="106" r="3.5" fill="#ffffff" />

        <!-- Right Eye -->
        <ellipse cx="142" cy="110" rx="14" ry="18" fill="#111827" stroke="#000" stroke-width="1.5" />
        <circle cx="142" cy="110" r="11" fill="url(#eyeGlow)" />
        <circle cx="139" cy="106" r="3.5" fill="#ffffff" />
      {/if}

      <!-- Goblin Nose (Wide & Hooked) -->
      <path d="M 115,118 Q 120,132 125,118" stroke="#064e3b" stroke-width="3" fill="none" stroke-linecap="round" />

      <!-- Mouth & Fangs -->
      {#if mood === 'Feral'}
        <!-- Open Feral Snarl -->
        <path d="M 90,140 Q 120,165 150,140 Z" fill="#450a0a" stroke="#7f1d1d" stroke-width="2" />
        <!-- Top Fangs -->
        <polygon points="98,140 104,148 110,140" fill="#ffffff" />
        <polygon points="130,140 136,148 142,140" fill="#ffffff" />
      {:else if mood === 'Exhausted'}
        <!-- Droopy Tired Mouth -->
        <path d="M 100,150 Q 120,142 140,150" stroke="#064e3b" stroke-width="3.5" fill="none" stroke-linecap="round" />
      {:else}
        <!-- Grin with single cute Goblin Tusk -->
        <path d="M 95,142 Q 120,160 145,142" stroke="#064e3b" stroke-width="3.5" fill="none" stroke-linecap="round" />
        <!-- Tusk pointing up from bottom jaw -->
        <polygon points="104,148 108,138 113,148" fill="#ffffff" stroke="#cbd5e1" stroke-width="1" />
      {/if}

      <!-- Goblin Body & Torso -->
      <path d="M 80,165 Q 120,175 160,165 L 175,225 Q 120,235 65,225 Z" fill="#064e3b" stroke="#022c22" stroke-width="3" />
      
      <!-- Leather Harness / Armor Strap -->
      <path d="M 82,166 L 160,225" stroke="#78350f" stroke-width="7" stroke-linecap="round" />
      <circle cx="121" cy="195" r="6" fill="#f59e0b" stroke="#78350f" stroke-width="2" />
    </svg>
  </div>

  <!-- Goblin Title & Badge -->
  <div class="mt-2 flex flex-col items-center">
    <div class="flex items-center gap-2">
      <span class="text-lg md:text-xl font-bold tracking-wide text-zinc-100">{goblinState.profile?.name || 'Gribble'}</span>
      <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
        Lv. {level}
      </span>
    </div>
    <span class="text-xs text-zinc-400 font-medium tracking-wider uppercase mt-0.5">{rank} • {mood}</span>
  </div>
</div>
