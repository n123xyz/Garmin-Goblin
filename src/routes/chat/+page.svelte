<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { goblinState, loadGoblinProfile, setGoblinSpeech } from '$lib/state/goblin.svelte';
  import { biometricsState, loadLatestBiometrics } from '$lib/state/biometrics.svelte';
  import type { ChatMessage, ChatResponse } from '$lib/types/garmin';
  import { Send, Trash2, Volume2, VolumeX, Sparkles, MessageSquare, Bot, User, HeartHandshake, Lock } from 'lucide-svelte';
  import toast from 'svelte-french-toast';

  let isTherapyMode = $state(false);
  let messages = $state<ChatMessage[]>([]);
  let therapyMessages = $state<ChatMessage[]>([]);
  let inputPrompt = $state('');
  let isSending = $state(false);
  let isMuted = $state(false);
  let chatScrollContainer: HTMLDivElement | null = $state(null);

  let goblin = $derived(goblinState.profile);
  let bio = $derived(biometricsState.current);

  const coachingPrompts = [
    "How is my Body Battery and recovery looking?",
    "Can I crush a heavy workout today?",
    "Roast my sleep schedule!",
    "Give me a tough-love goblin motivational speech!",
  ];

  const therapyPrompts = [
    "I am feeling overwhelmed and anxious right now.",
    "Help me untangle and reframe an anxious thought.",
    "My body feels tense and exhausted today.",
    "Let's do a gentle mental decompression check-in.",
  ];

  let currentPrompts = $derived(isTherapyMode ? therapyPrompts : coachingPrompts);
  let currentDisplayMessages = $derived(isTherapyMode ? therapyMessages : messages);

  onMount(async () => {
    await loadGoblinProfile();
    await loadLatestBiometrics();
    await loadChatHistory();
    initTherapyGreeting();
  });

  function initTherapyGreeting() {
    if (therapyMessages.length === 0) {
      therapyMessages = [
        {
          role: 'assistant',
          content: `Welcome to the Cavern Sanctuary. I am ${goblin?.name || 'Gribble'}, your Goblin Counselor. In this mode, conversations are 100% ephemeral and never saved to history. Your Body Battery is at ${bio?.body_battery || 82}% and your Stress Index is ${bio?.stress_level || 24}. Take a deep breath—what would you like to unpack?`,
          goblinMood: 'Zen',
        },
      ];
    }
  }

  async function loadChatHistory() {
    try {
      const history = await invoke<any[]>('get_chat_history');
      if (history && Array.isArray(history)) {
        messages = history.map(h => ({
          role: h.role,
          content: h.content,
          goblinMood: h.goblinMood,
          audioBase64: h.audioBase64,
        }));
      }
      if (messages.length === 0) {
        messages = [
          {
            role: 'assistant',
            content: `GRAH! Welcome to my cavern, human! I am ${goblin?.name || 'Gribble'}, your personal Garmin Goblin coach! Your Body Battery is at ${bio?.body_battery || 82}% and your Stress is ${bio?.stress_level || 24}. Ask me anything or let's get to work!`,
            goblinMood: 'Energetic',
          }
        ];
      }
      await scrollToBottom();
    } catch (e) {
      console.error('Failed to load chat history:', e);
    }
  }

  async function scrollToBottom() {
    await tick();
    if (chatScrollContainer) {
      chatScrollContainer.scrollTop = chatScrollContainer.scrollHeight;
    }
  }

  async function sendMessage(text?: string) {
    const promptToSend = text || inputPrompt.trim();
    if (!promptToSend || isSending) return;

    const userMsg: ChatMessage = { role: 'user', content: promptToSend };
    inputPrompt = '';
    isSending = true;

    if (isTherapyMode) {
      // In Therapy Mode: STRICTLY IN-MEMORY, NEVER SAVE TO DATABASE
      therapyMessages = [...therapyMessages, userMsg];
      await scrollToBottom();

      try {
        const response = await invoke<ChatResponse>('chat_with_therapy_goblin', {
          history: therapyMessages,
        });

        const goblinMsg: ChatMessage = {
          role: 'assistant',
          content: response.response,
          goblinMood: response.goblin_mood,
        };

        therapyMessages = [...therapyMessages, goblinMsg];
        setGoblinSpeech(response.response.slice(0, 100) + '...', 5000);
        await scrollToBottom();
      } catch (e) {
        toast.error('Failed to chat in therapy mode: ' + e);
      } finally {
        isSending = false;
      }
    } else {
      // Standard Coaching Mode: Saved to DB
      messages = [...messages, userMsg];
      await scrollToBottom();

      try {
        await invoke('save_chat_message', {
          role: 'user',
          content: promptToSend,
          goblinMood: null,
          audioBase64: null,
        });

        const response = await invoke<ChatResponse>('chat_with_goblin', {
          history: messages,
          imageUri: null,
          audioBase64: null,
          mode: null,
        });

        const goblinMsg: ChatMessage = {
          role: 'assistant',
          content: response.response,
          goblinMood: response.goblin_mood,
        };

        messages = [...messages, goblinMsg];
        setGoblinSpeech(response.response.slice(0, 100) + '...', 5000);

        await invoke('save_chat_message', {
          role: 'assistant',
          content: response.response,
          goblinMood: response.goblin_mood,
          audioBase64: null,
        });

        await scrollToBottom();
      } catch (e) {
        toast.error('Failed to chat: ' + e);
      } finally {
        isSending = false;
      }
    }
  }

  async function clearHistory() {
    if (isTherapyMode) {
      therapyMessages = [];
      initTherapyGreeting();
      toast.success('Therapy chat memory wiped clean.');
    } else {
      try {
        await invoke('clear_chat_history');
        messages = [
          {
            role: 'assistant',
            content: `Chat history cleared! The slate is wiped clean. What are our orders, warrior?`,
            goblinMood: 'Neutral',
          }
        ];
        toast.success('History cleared');
      } catch (e) {
        toast.error('Failed to clear history: ' + e);
      }
    }
  }
</script>

<div class="max-w-4xl mx-auto p-4 md:p-6 w-full flex flex-col h-full flex-1">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between border-b border-zinc-800 pb-3 mb-3 gap-2">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-2xl {isTherapyMode ? 'bg-teal-500/20 border-teal-500/40' : 'bg-emerald-500/20 border-emerald-500/40'} border flex items-center justify-center text-xl transition-all">
        {isTherapyMode ? '🌿' : '👹'}
      </div>
      <div>
        <h2 class="text-base md:text-lg font-bold text-zinc-100 flex items-center gap-2">
          {isTherapyMode ? `Counselor ${goblin?.name || 'Gribble'}` : `Chat with ${goblin?.name || 'Gribble'}`}
          <span class="text-xs px-2 py-0.5 rounded-full {isTherapyMode ? 'bg-teal-500/20 text-teal-300' : 'bg-emerald-500/20 text-emerald-300'} font-mono">
            {isTherapyMode ? 'Therapy Mode' : 'Gemma 4 IT'}
          </span>
        </h2>
        <span class="text-xs text-zinc-400">
          {isTherapyMode ? 'Confidential Therapist' : 'Coach'}
        </span>
      </div>
    </div>

    <div class="flex items-center gap-2 self-end sm:self-center">
      <!-- Mode Toggle -->
      <div class="flex items-center bg-zinc-950 p-1 rounded-xl border border-zinc-800 text-xs">
        <button
          onclick={() => (isTherapyMode = false)}
          class="px-2.5 py-1 rounded-lg font-semibold transition-all cursor-pointer {!isTherapyMode ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Coaching 👹
        </button>
        <button
          onclick={() => { isTherapyMode = true; initTherapyGreeting(); }}
          class="px-2.5 py-1 rounded-lg font-semibold transition-all cursor-pointer {isTherapyMode ? 'bg-teal-500/20 text-teal-300 border border-teal-500/40' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Therapy 🌿
        </button>
      </div>

      <button
        onclick={clearHistory}
        class="w-8 h-8 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-rose-400 flex items-center justify-center transition-colors cursor-pointer shrink-0"
        title={isTherapyMode ? 'Clear Memory' : 'Clear History'}
      >
        <Trash2 class="w-4 h-4" />
      </button>
    </div>
  </div>

  <!-- Ephemeral Mode Privacy Notice (Therapy Mode only) -->
  {#if isTherapyMode}
    <div class="mb-2 px-3 py-1.5 rounded-xl bg-teal-950/20 border border-teal-500/30 flex items-center justify-between text-xs text-teal-300/90 shadow-sm animate-in fade-in">
      <div class="flex items-center gap-1.5">
        <Lock class="w-3.5 h-3.5 text-teal-400 shrink-0" />
        <span>In-memory only (never saved).</span>
      </div>
      <a href="/therapy" class="text-teal-400 hover:text-teal-200 underline text-[11px] shrink-0 ml-2">
        Therapy Hub →
      </a>
    </div>
  {/if}

  <!-- Messages Scroll Area -->
  <div 
    bind:this={chatScrollContainer}
    class="flex-1 overflow-y-auto space-y-4 pr-1 pb-4"
  >
    {#each currentDisplayMessages as msg}
      {#if msg.role === 'user'}
        <div class="flex items-end justify-end gap-2.5">
          <div class="max-w-lg {isTherapyMode ? 'bg-teal-600/30 border border-teal-500/40' : 'bg-emerald-600/90'} text-zinc-100 p-3.5 rounded-2xl rounded-br-sm shadow-md text-xs md:text-sm font-medium leading-relaxed">
            {msg.content}
          </div>
          <div class="w-7 h-7 rounded-full {isTherapyMode ? 'bg-teal-700' : 'bg-emerald-700'} flex items-center justify-center text-zinc-100 text-xs shrink-0 font-bold">
            <User class="w-4 h-4" />
          </div>
        </div>
      {:else}
        <div class="flex items-start gap-2.5">
          <div class="w-8 h-8 rounded-xl bg-zinc-900 border {isTherapyMode ? 'border-teal-500/40 text-teal-300' : 'border-emerald-500/40'} flex items-center justify-center text-base shrink-0 shadow-[0_0_10px_rgba(20,184,166,0.3)]">
            {isTherapyMode ? '🌿' : '👹'}
          </div>
          <div class="max-w-xl bg-zinc-900/95 border border-zinc-800/80 text-zinc-200 p-4 rounded-2xl rounded-tl-sm shadow-md text-xs md:text-sm leading-relaxed space-y-1">
            <p class="whitespace-pre-wrap">{msg.content}</p>
          </div>
        </div>
      {/if}
    {/each}

    {#if isSending}
      <div class="flex items-center gap-2 text-xs {isTherapyMode ? 'text-teal-400' : 'text-emerald-400'} italic animate-pulse">
        <span>{isTherapyMode ? '🌿' : '👹'}</span>
        <span>{goblin?.name || 'Gribble'} {isTherapyMode ? 'is mindfully reflecting...' : 'is plotting response...'}</span>
      </div>
    {/if}
  </div>

  <!-- Quick Prompts Pills -->
  <div class="flex items-center gap-2 overflow-x-auto py-2 mb-2 no-scrollbar">
    {#each currentPrompts as q}
      <button
        onclick={() => sendMessage(q)}
        class="shrink-0 text-xs px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 {isTherapyMode ? 'hover:border-teal-500/40 text-teal-300/90' : 'hover:border-emerald-500/40 text-zinc-300'} transition-colors cursor-pointer"
      >
        {q}
      </button>
    {/each}
  </div>

  <!-- Chat Input Form -->
  <form 
    onsubmit={(e) => { e.preventDefault(); sendMessage(); }}
    class="flex items-center gap-2 bg-zinc-900/90 border border-zinc-800 rounded-2xl p-2 shadow-xl"
  >
    <input
      type="text"
      placeholder={isTherapyMode ? 'Speak openly with your counselor (confidential & in-memory only)...' : 'Ask your Goblin about workouts, sleep, or nutrition...'}
      bind:value={inputPrompt}
      disabled={isSending}
      class="flex-1 bg-transparent px-3 py-2 text-xs md:text-sm text-zinc-100 focus:outline-none placeholder:text-zinc-500"
    />

    <button
      type="submit"
      disabled={!inputPrompt.trim() || isSending}
      class="w-10 h-10 rounded-xl {isTherapyMode ? 'bg-teal-500 hover:bg-teal-400' : 'bg-emerald-500 hover:bg-emerald-400'} disabled:opacity-40 text-zinc-950 flex items-center justify-center transition-all cursor-pointer shrink-0 font-bold"
    >
      <Send class="w-4 h-4" />
    </button>
  </form>
</div>
