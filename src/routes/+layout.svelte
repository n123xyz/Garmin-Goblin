<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { Toaster } from 'svelte-french-toast';
  import { loadGoblinProfile, loadQuests, goblinState } from '$lib/state/goblin.svelte';
  import { loadLatestBiometrics, biometricsState } from '$lib/state/biometrics.svelte';
  import { loadSettings } from '$lib/state/settings.svelte';
  import { checkOllamaHealth } from '$lib/state/ollama.svelte';
  import { startNotificationTicker, loadNotificationSettings } from '$lib/services/notifications.svelte';
  import { Castle, Activity, ScrollText, MessageSquare, Settings, Coins, Zap, Sparkles, Moon, Calendar, Utensils, Brain, HeartHandshake } from 'lucide-svelte';

  let { children } = $props();

  let currentPath = $derived($page?.url?.pathname || '/');
  let gold = $derived(goblinState.profile?.gold || 0);
  let energy = $derived(goblinState.profile?.energy || 85);
  let level = $derived(goblinState.profile?.level || 1);

  onMount(async () => {
    try {
      loadNotificationSettings();
      await loadSettings();
      await loadGoblinProfile();
      await loadLatestBiometrics();
      await loadQuests();
      startNotificationTicker(
        () => biometricsState.current,
        () => goblinState.quests
      );
      const isAndroid = typeof window !== 'undefined' && (!!(window as any).__TAURI_INTERNALS__ || /Android/i.test(navigator.userAgent));
      if (!isAndroid) {
        checkOllamaHealth();
      }
    } catch (err) {
      console.error('Error during initial layout mount:', err);
    }
  });
</script>

<Toaster position="top-center" toastOptions={{ duration: 3500 }} />

<div class="h-[100dvh] w-screen overflow-hidden bg-[#090d10] text-zinc-100 selection:bg-emerald-500/30 flex flex-col-reverse md:flex-col">
  <!-- Top / Bottom Navigation Bar -->
  <header class="h-16 shrink-0 border-t md:border-t-0 md:border-b border-zinc-800/80 bg-zinc-950/95 backdrop-blur-md flex items-center px-2 md:px-6 justify-between z-40 overflow-x-auto no-scrollbar">
    <!-- App Logo & RPG Goblin Summary (Desktop) -->
    <div class="hidden md:flex items-center gap-4">
      <a href="/" class="flex items-center gap-2.5 group">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-emerald-500 to-emerald-700 flex items-center justify-center font-black text-zinc-950 text-xl shadow-[0_0_15px_rgba(16,185,129,0.4)] group-hover:scale-105 transition-transform">
          👹
        </div>
        <div>
          <span class="text-base font-black tracking-tight text-zinc-100 flex items-center gap-1.5">
            Garmin<span class="text-emerald-400">Goblin</span>
          </span>
          <span class="text-[10px] text-zinc-400 font-bold uppercase tracking-wider block -mt-1">On-Device AI Companion</span>
        </div>
      </a>

      <!-- Quick RPG HUD Stats -->
      <div class="flex items-center gap-3 bg-zinc-900/80 px-3 py-1.5 rounded-xl border border-zinc-800/80 text-xs font-bold">
        <div class="flex items-center gap-1 text-amber-400">
          <Coins class="w-3.5 h-3.5" />
          <span>{gold}</span>
        </div>
        <div class="w-px h-3 bg-zinc-700"></div>
        <div class="flex items-center gap-1 text-emerald-400">
          <Zap class="w-3.5 h-3.5" />
          <span>{energy}% BB</span>
        </div>
        <div class="w-px h-3 bg-zinc-700"></div>
        <div class="text-zinc-300 font-mono">
          Lv. {level}
        </div>
      </div>
    </div>

    <!-- Navigation Links -->
    <nav class="flex items-center gap-1 md:gap-2.5 w-full md:w-auto justify-between md:justify-end shrink-0 min-w-max px-1">
      <a 
        href="/" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Castle class="w-4 h-4 md:w-4 md:h-4" />
        <span>Cavern</span>
      </a>

      <a 
        href="/biometrics" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/biometrics' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Activity class="w-4 h-4 md:w-4 md:h-4" />
        <span>Biometrics</span>
      </a>

      <a 
        href="/sleep" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/sleep' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Moon class="w-4 h-4 md:w-4 md:h-4" />
        <span>Sleep</span>
      </a>

      <a 
        href="/yoga" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/yoga' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Sparkles class="w-4 h-4 md:w-4 md:h-4" />
        <span>Yoga</span>
      </a>

      <a 
        href="/meditation" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/meditation' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Moon class="w-4 h-4 md:w-4 md:h-4" />
        <span>Meditation</span>
      </a>

      <a 
        href="/therapy" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/therapy' ? 'text-teal-400 bg-teal-500/10 border border-teal-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <HeartHandshake class="w-4 h-4 md:w-4 md:h-4" />
        <span>Therapy</span>
      </a>

      <a 
        href="/journal" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/journal' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Calendar class="w-4 h-4 md:w-4 md:h-4" />
        <span>Journal</span>
      </a>

      <a 
        href="/quests" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/quests' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <ScrollText class="w-4 h-4 md:w-4 md:h-4" />
        <span>Quests</span>
      </a>

      <a 
        href="/chat" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/chat' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <MessageSquare class="w-4 h-4 md:w-4 md:h-4" />
        <span>Chat</span>
      </a>

      <a 
        href="/vision" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/vision' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Utensils class="w-4 h-4 md:w-4 md:h-4" />
        <span>Feed Gribble</span>
      </a>

      <a 
        href="/carit" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/carit' ? 'text-indigo-400 bg-indigo-500/10 border border-indigo-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Brain class="w-4 h-4 md:w-4 md:h-4" />
        <span>Focus</span>
      </a>

      <a 
        href="/facename" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/facename' ? 'text-purple-400 bg-purple-500/10 border border-purple-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Sparkles class="w-4 h-4 md:w-4 md:h-4" />
        <span>Memory</span>
      </a>

      <a 
        href="/vismotor" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/vismotor' ? 'text-cyan-400 bg-cyan-500/10 border border-cyan-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Zap class="w-4 h-4 md:w-4 md:h-4" />
        <span>Visuomotor</span>
      </a>

      <a 
        href="/settings" 
        class="flex flex-col md:flex-row items-center gap-1 md:gap-2 px-3 py-1.5 rounded-xl text-xs md:text-sm font-semibold transition-all {currentPath === '/settings' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Settings class="w-4 h-4 md:w-4 md:h-4" />
        <span>Settings</span>
      </a>
    </nav>
  </header>

  <!-- Main View Container -->
  <main class="flex-1 overflow-y-auto relative flex flex-col">
    {#if children}
      {@render children()}
    {/if}
  </main>
</div>
