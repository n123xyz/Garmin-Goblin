<script lang="ts">
  import { page } from '$app/stores';
  import { ShieldAlert, RefreshCw, Castle } from 'lucide-svelte';
</script>

<div class="max-w-xl mx-auto p-6 my-auto text-center space-y-5">
  <div class="w-16 h-16 rounded-3xl bg-rose-500/20 text-rose-400 border border-rose-500/40 flex items-center justify-center text-3xl mx-auto shadow-[0_0_20px_rgba(244,63,94,0.3)]">
    <ShieldAlert class="w-8 h-8" />
  </div>
  <div>
    <h1 class="text-2xl md:text-3xl font-black text-zinc-100">
      {$page?.status || 500} - {$page?.error?.message || 'Internal Error'}
    </h1>
    <p class="text-xs md:text-sm text-zinc-400 mt-1">
      An error occurred during page rendering or biometric state hydration.
    </p>
  </div>

  <div class="flex items-center justify-center gap-3 pt-2">
    <a
      href="/"
      class="px-4 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-black text-xs md:text-sm flex items-center gap-1.5 transition-all shadow-[0_0_15px_rgba(16,185,129,0.3)]"
    >
      <Castle class="w-4 h-4" />
      <span>Return to Cavern</span>
    </a>
    <button
      onclick={() => window.location.reload()}
      class="px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-bold text-xs md:text-sm flex items-center gap-1.5 transition-all border border-zinc-700 cursor-pointer"
    >
      <RefreshCw class="w-4 h-4" />
      <span>Reload</span>
    </button>
  </div>
</div>
