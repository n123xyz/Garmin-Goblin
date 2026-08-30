<script lang="ts">
  import { onMount } from 'svelte';
  import { goblinState, loadGoblinProfile, loadQuests } from '$lib/state/goblin.svelte';
  import { biometricsState, loadLatestBiometrics, loadHealthInsights, loadActivities } from '$lib/state/biometrics.svelte';
  import { caritState, loadTodayCaritSummary } from '$lib/state/carit.svelte';
  import { facenameState, loadTodayFaceNameSummary } from '$lib/state/facename.svelte';
  import { vismotorState, loadTodayVismotorSummary } from '$lib/state/vismotor.svelte';
  import GoblinAvatar from '$lib/components/GoblinAvatar.svelte';
  import BiometricGauge from '$lib/components/BiometricGauge.svelte';
  import MedGemmaCard from '$lib/components/MedGemmaCard.svelte';
  import QuestCard from '$lib/components/QuestCard.svelte';
  import { Zap, Footprints, Moon, ShieldAlert, Sparkles, MessageSquare, Utensils, ChevronRight, Activity, Flame, Heart, Timer, Brain, Compass } from 'lucide-svelte';

  let bio = $derived(biometricsState.current);
  let goblin = $derived(goblinState.profile);
  let activities = $derived(biometricsState.activities || []);
  let activeQuests = $derived((goblinState.quests || []).filter(q => !q.is_claimed).slice(0, 2));
  let todayCarit = $derived(caritState.todaySummary);
  let todayFaceName = $derived(facenameState.todaySummary);
  let todayVismotor = $derived(vismotorState.todaySummary);

  onMount(async () => {
    try {
      await loadGoblinProfile();
      await loadLatestBiometrics();
      await loadActivities(10);
      await loadQuests();
      await loadHealthInsights();
      await loadTodayCaritSummary();
      await loadTodayFaceNameSummary();
      await loadTodayVismotorSummary();
    } catch (e) {
      console.error('Error loading page biometrics:', e);
    }
  });
</script>

<div class="max-w-6xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Top Hero Header with Level Bar -->
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-emerald-950/30 border border-emerald-500/20 rounded-3xl p-4 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(16,185,129,0.3)]">
        👹
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          {goblin?.name || 'Gribble'}'s Cavern
          <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/40">
            {goblin?.evolution_rank || 'Cave Scamp'}
          </span>
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">Your physical biometrics fuel this on-device goblin creature.</p>
      </div>
    </div>

    <!-- XP Progress Bar -->
    <div class="w-full md:w-64 bg-zinc-950/80 p-3 rounded-2xl border border-zinc-800">
      <div class="flex items-center justify-between text-xs font-bold text-zinc-400 mb-1.5">
        <span>Level {goblin?.level || 1} XP</span>
        <span class="font-mono text-emerald-400">{goblin?.xp || 0} / {goblin?.xp_to_next_level || 500}</span>
      </div>
      <div class="w-full bg-zinc-900 h-2.5 rounded-full overflow-hidden border border-zinc-800">
        <div 
          class="h-full bg-gradient-to-r from-emerald-500 to-emerald-400 rounded-full transition-all duration-500"
          style="width: {Math.min(Math.round(((goblin?.xp || 0) / (goblin?.xp_to_next_level || 500)) * 100), 100)}%;"
        ></div>
      </div>
    </div>
  </div>

  <!-- Central Split: Interactive Goblin Pet & Core Biometric Gauges -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- Goblin Avatar Hero Chamber -->
    <div class="lg:col-span-5 bg-gradient-to-b from-zinc-900 via-zinc-900 to-emerald-950/20 border border-emerald-500/30 rounded-3xl p-6 shadow-2xl flex flex-col items-center justify-between relative overflow-hidden min-h-[380px]">
      <!-- Background Ambient Mist -->
      <div class="absolute -bottom-10 left-1/2 -translate-x-1/2 w-64 h-32 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none"></div>

      <!-- Quick Action Pill Top -->
      <div class="w-full flex items-center justify-between z-10">
        <span class="text-xs px-3 py-1 rounded-full font-bold bg-zinc-800/80 text-zinc-300 border border-zinc-700">
          State: {goblin?.mood || 'Energetic'}
        </span>

        <a 
          href="/biometrics"
          class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-semibold border border-zinc-700 transition-colors"
        >
          <Activity class="w-3.5 h-3.5 text-emerald-400" />
          <span>Biometrics & BLE</span>
        </a>
      </div>

      <!-- Goblin Pet Avatar Component -->
      <div class="my-auto z-10">
        <GoblinAvatar />
      </div>

      <!-- Quick Action Buttons -->
      <div class="grid grid-cols-2 gap-3 w-full mt-4 z-10">
        <a 
          href="/chat"
          class="flex items-center justify-center gap-2 p-2.5 rounded-xl bg-emerald-500/15 hover:bg-emerald-500/25 border border-emerald-500/30 text-emerald-300 text-xs font-bold transition-all"
        >
          <MessageSquare class="w-4 h-4" />
          <span>Talk with Gribble</span>
        </a>

        <a 
          href="/vision"
          class="flex items-center justify-center gap-2 p-2.5 rounded-xl bg-amber-500/15 hover:bg-amber-500/25 border border-amber-500/30 text-amber-300 text-xs font-bold transition-all"
        >
          <Utensils class="w-4 h-4" />
          <span>Feed Gribble</span>
        </a>
      </div>
    </div>

    <!-- Right Side: Garmin Live Biometrics Grid -->
    <div class="lg:col-span-7 flex flex-col gap-4">
      <div class="grid grid-cols-2 sm:grid-cols-2 gap-3.5">
        <BiometricGauge
          label="Body Battery"
          value={bio?.body_battery || 0}
          max={100}
          unit="%"
          subtitle={bio && bio.body_battery > 0 ? "Energy Reserves" : "No Sync Data"}
          color="emerald"
          icon="⚡"
        />

        <BiometricGauge
          label="Daily Steps"
          value={bio?.steps || 0}
          max={bio?.step_goal || 10000}
          unit={`/ ${bio?.step_goal || 10000}`}
          subtitle={bio && bio.steps > 0 ? `${Math.round(((bio.steps) / (bio?.step_goal || 10000)) * 100)}% Goal Reached` : "0% Goal Reached"}
          color="cyan"
          icon="👟"
        />

        <BiometricGauge
          label="Stress Level"
          value={bio?.stress_level || 0}
          max={100}
          subtitle={bio && bio.stress_level > 0 ? (bio.stress_level < 35 ? 'Low / Restful' : 'High Autonomic Load') : "No Sync Data"}
          color={bio && bio.stress_level > 50 ? 'rose' : 'emerald'}
          icon="🧘"
        />

        <a href="/sleep" class="block group">
          <BiometricGauge
            label="Sleep Score"
            value={bio?.sleep_score || 0}
            max={100}
            subtitle={bio && bio.sleep_score > 0 ? (bio.sleep_score >= 80 ? "Excellent Recovery" : "Fair Recovery") : "No Sync Data"}
            color="purple"
            icon="🌙"
          />
        </a>
      </div>

      <!-- Sleep Architecture & Advisor Quick Banner -->
      <div class="bg-gradient-to-r from-indigo-950/40 via-zinc-900 to-purple-950/30 border border-indigo-500/30 hover:border-indigo-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 flex items-center justify-center text-lg shadow-[0_0_12px_rgba(99,102,241,0.2)]">
            🌙
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">Sleep Architecture & Advisor</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 uppercase tracking-wider">
                Circadian AI
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">Deep & REM hypnograms, multi-night history, sleep debt, and bedtime coaching.</span>
          </div>
        </div>

        <a 
          href="/sleep" 
          class="px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-zinc-100 font-bold text-xs flex items-center gap-1.5 transition-all shadow-md shadow-indigo-600/30 shrink-0"
        >
          <span>Sleep Advisor</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- CARIT Cognitive Readiness Quick Banner -->
      <div class="bg-gradient-to-r from-indigo-950/40 via-zinc-900 to-indigo-950/30 border border-indigo-500/30 hover:border-indigo-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 flex items-center justify-center text-lg shadow-[0_0_12px_rgba(99,102,241,0.2)]">
            🧠
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">CARIT Motor Inhibition</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold {todayCarit?.hasCompletedToday ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40'} uppercase tracking-wider">
                {todayCarit?.hasCompletedToday ? `Score ${todayCarit.latestScore}/100` : 'Daily Test'}
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">
              {todayCarit?.hasCompletedToday 
                ? `Last test: ${todayCarit.latestMeanRtMs?.toFixed(0)}ms RT • ${((todayCarit.latestInhibitionAcc || 0) * 100).toFixed(0)}% inhibition control` 
                : 'Test your motor reaction speed and executive inhibitory control (~75s).'}
            </span>
          </div>
        </div>

        <a 
          href="/carit" 
          class="px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-zinc-100 font-bold text-xs flex items-center gap-1.5 transition-all shadow-md shadow-indigo-600/30 shrink-0"
        >
          <span>{todayCarit?.hasCompletedToday ? 'Retest' : 'Start Test'}</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- FACENAME Associative Memory Quick Banner -->
      <div class="bg-gradient-to-r from-purple-950/40 via-zinc-900 to-purple-950/30 border border-purple-500/30 hover:border-purple-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-purple-500/20 text-purple-300 border border-purple-500/40 flex items-center justify-center text-lg shadow-[0_0_12px_rgba(168,85,247,0.2)]">
            🧩
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">FACENAME Associative Memory</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold {todayFaceName?.hasCompletedToday ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-purple-500/20 text-purple-300 border border-purple-500/40'} uppercase tracking-wider">
                {todayFaceName?.hasCompletedToday ? `Score ${todayFaceName.latestScore}/100` : 'Daily Test'}
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">
              {todayFaceName?.hasCompletedToday 
                ? `Last test: ${((todayFaceName.latestRecallAccuracy || 0) * 100).toFixed(0)}% Recall • ${todayFaceName.latestRecallRtMs?.toFixed(0)}ms retrieval latency` 
                : 'Study face-name pairs, complete speed distractor, and test recall (~90s).'}
            </span>
          </div>
        </div>

        <a 
          href="/facename" 
          class="px-3.5 py-1.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-zinc-100 font-bold text-xs flex items-center gap-1.5 transition-all shadow-md shadow-purple-600/30 shrink-0"
        >
          <span>{todayFaceName?.hasCompletedToday ? 'Retest' : 'Start Test'}</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- VISMOTOR Visuomotor Speed Quick Banner -->
      <div class="bg-gradient-to-r from-cyan-950/40 via-zinc-900 to-cyan-950/30 border border-cyan-500/30 hover:border-cyan-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 flex items-center justify-center text-lg shadow-[0_0_12px_rgba(6,182,212,0.2)]">
            ⚡
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">VISMOTOR Choice Reaction</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold {todayVismotor?.hasCompletedToday ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'} uppercase tracking-wider">
                {todayVismotor?.hasCompletedToday ? `Score ${todayVismotor.latestScore}/100` : 'Daily Test'}
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">
              {todayVismotor?.hasCompletedToday 
                ? `Last test: ${todayVismotor.latestMeanRtMs?.toFixed(0)}ms RT • ${((todayVismotor.latestAccuracy || 0) * 100).toFixed(0)}% choice accuracy` 
                : 'Test visual processing speed and left vs right spatial choice reaction (~60s).'}
            </span>
          </div>
        </div>

        <a 
          href="/vismotor" 
          class="px-3.5 py-1.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-zinc-100 font-bold text-xs flex items-center gap-1.5 transition-all shadow-md shadow-cyan-600/30 shrink-0"
        >
          <span>{todayVismotor?.hasCompletedToday ? 'Retest' : 'Start Test'}</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- Quick Biometric Summary Banner -->
      <div class="bg-zinc-900/90 border border-zinc-800 rounded-2xl p-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold">
            💓
          </div>
          <div>
            <span class="text-xs text-zinc-400 font-semibold block">Heart Rate & HRV Status</span>
            <span class="text-sm font-bold text-zinc-200">
              Resting: {bio?.resting_hr || 0} bpm • HRV: {bio?.hrv_status || 'No Data'} {bio && bio.hrv_rmssd > 0 ? `(${bio.hrv_rmssd} ms)` : ''}
            </span>
          </div>
        </div>

        <a 
          href="/biometrics" 
          class="flex items-center gap-1 text-xs text-emerald-400 font-bold hover:underline"
        >
          <span>Graphs</span>
          <ChevronRight class="w-4 h-4" />
        </a>
      </div>

      <!-- SuperTonic Guided Yoga Studio Quick Banner -->
      <div class="bg-gradient-to-r from-purple-950/40 via-zinc-900 to-emerald-950/30 border border-purple-500/30 hover:border-purple-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-purple-500/20 text-purple-300 border border-purple-500/40 flex items-center justify-center text-lg">
            🧘‍♀️
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">SuperTonic Guided Yoga</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-purple-500/20 text-purple-300 border border-purple-500/40 uppercase tracking-wider">
                8 Poses
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">Neural TTS spoken guidance, posture cues, and mindful flow.</span>
          </div>
        </div>

        <a 
          href="/yoga" 
          class="px-3.5 py-1.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-zinc-100 font-bold text-xs flex items-center gap-1.5 transition-all shadow-md shadow-purple-600/30 shrink-0"
        >
          <span>Start Flow</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- SuperTonic Guided Meditation Sanctuary Quick Banner -->
      <div class="bg-gradient-to-r from-cyan-950/40 via-zinc-900 to-emerald-950/30 border border-cyan-500/30 hover:border-cyan-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 flex items-center justify-center text-lg">
            🧘‍♂️
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">Guided Meditation Sanctuary</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 uppercase tracking-wider">
                4 Practices
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">Box breathing, 5-4-3-2-1 grounding, Leaves on a Stream, & Yoga Nidra body scan.</span>
          </div>
        </div>

        <a 
          href="/meditation" 
          class="px-3.5 py-1.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-zinc-950 font-black text-xs flex items-center gap-1.5 transition-all shadow-md shadow-cyan-600/30 shrink-0"
        >
          <span>Meditate</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- Habits, Food & Emotional Calendar Banner -->
      <div class="bg-gradient-to-r from-amber-950/30 via-zinc-900 to-indigo-950/30 border border-amber-500/30 hover:border-amber-500/50 rounded-2xl p-4 flex items-center justify-between transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-amber-500/20 text-amber-300 border border-amber-500/40 flex items-center justify-center text-lg">
            📅
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold text-zinc-100">Habits, Food & Mood Calendar</h4>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase tracking-wider">
                Daily Timeline
              </span>
            </div>
            <span class="text-xs text-zinc-400 mt-0.5 block">Log meals, macronutrients, subjective stress check-ins, and view monthly activity timeline.</span>
          </div>
        </div>

        <a 
          href="/journal" 
          class="px-3.5 py-1.5 rounded-xl bg-amber-500 hover:bg-amber-400 text-zinc-950 font-black text-xs flex items-center gap-1.5 transition-all shadow-md shadow-amber-500/30 shrink-0"
        >
          <span>Journal</span>
          <ChevronRight class="w-3.5 h-3.5" />
        </a>
      </div>

      <!-- Active Quests Preview -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-bold text-zinc-200 flex items-center gap-2">
            <span>📜</span> Active Goblin Quests
          </h3>
          <a href="/quests" class="text-xs text-amber-400 hover:underline font-bold">
            View All ({(goblinState.quests || []).length})
          </a>
        </div>

        <div class="space-y-2.5">
          {#each activeQuests as quest (quest.id)}
            <QuestCard {quest} />
          {/each}
        </div>
      </div>

      <!-- Recent Garmin Workouts & Activities -->
      {#if activities && activities.length > 0}
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-bold text-zinc-200 flex items-center gap-2">
              <span class="text-emerald-400">⚡</span> Synced Garmin Workouts ({activities.length})
            </h3>
            <a href="/biometrics" class="text-xs text-emerald-400 hover:underline font-bold">
              View In Depth
            </a>
          </div>

          <div class="space-y-2">
            {#each activities.slice(0, 3) as act (act.id)}
              <div class="bg-zinc-900/90 border border-zinc-800 hover:border-emerald-500/40 rounded-2xl p-3.5 flex items-center justify-between transition-all">
                <div class="flex items-center gap-3">
                  <div class="w-9 h-9 rounded-xl bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center justify-center font-bold text-sm">
                    {act.activity_type.toLowerCase().includes('run') ? '🏃' : act.activity_type.toLowerCase().includes('cyc') || act.activity_type.toLowerCase().includes('bike') ? '🚴' : act.activity_type.toLowerCase().includes('walk') ? '🚶' : '⚡'}
                  </div>
                  <div>
                    <h4 class="text-sm font-bold text-zinc-200">{act.title || act.activity_type}</h4>
                    <div class="flex items-center gap-3 text-xs text-zinc-400 mt-0.5">
                      <span>{act.start_time.substring(0, 16)}</span>
                      {#if act.distance_meters > 0}
                        <span>• {(act.distance_meters / 1000).toFixed(2)} km</span>
                      {/if}
                      <span>• {Math.round(act.duration_sec / 60)} min</span>
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-3 text-right">
                  <div>
                    <span class="text-xs font-bold text-amber-400 flex items-center gap-1 justify-end">
                      <Flame class="w-3 h-3" /> {act.calories} kcal
                    </span>
                    <span class="text-[11px] text-rose-400 flex items-center gap-1 justify-end font-semibold mt-0.5">
                      <Heart class="w-3 h-3" /> {act.avg_hr} bpm
                    </span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Bottom: MedGemma Biometric Health Coach Assessment -->
  <MedGemmaCard />
</div>
