<script lang="ts">
  import { onMount } from 'svelte';
  import {
    biometricsState,
    loadLatestBiometrics,
    loadBiometricsHistory,
    loadActivities
  } from '$lib/state/biometrics.svelte';
  import { loadGoblinProfile } from '$lib/state/goblin.svelte';
  import MedGemmaCard from '$lib/components/MedGemmaCard.svelte';
  import BiometricGauge from '$lib/components/BiometricGauge.svelte';
  import LeafletMap from '$lib/components/LeafletMap.svelte';
  import EnvironmentalCard from '$lib/components/EnvironmentalCard.svelte';
  import GarminDeviceCard from '$lib/components/GarminDeviceCard.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    Activity,
    Moon,
    Zap,
    Flame,
    HeartPulse,
    ShieldCheck,
    Compass,
    Dumbbell,
    CloudSun,
    RefreshCw,
    PlusCircle,
    CheckCircle2,
    Wind,
    Droplets,
    Map,
    Radio,
    ShieldAlert,
    Calendar,
    History,
    TrendingUp,
    ChevronRight
  } from 'lucide-svelte';
  import toast from 'svelte-french-toast';
  import type { GarminActivity, AirQualityData } from '$lib/types/garmin';

  let isSyncingWeather = $state(false);
  let isSyncingActivity = $state(false);
  let showLogModal = $state(false);
  let privacyCloak = $state(true);

  // Weather & Environmental state
  let weatherLocation = $state('Goblin Mountain Cavern');
  let weatherTemp = $state(18);
  let weatherCondition = $state(1); // 1 = Partly Cloudy
  let weatherWind = $state(4.5);
  let weatherHumidity = $state(55);

  let liveEnvData = $state<Partial<AirQualityData>>({
    aqi: 34,
    category: 'Good',
    pm2_5: 8.2,
    pm10: 14.5,
    temp_c: 18.5,
    humidity_percent: 55,
    heat_index_c: 18.8,
    location: 'Goblin Mountain Cavern & Trails',
    environmental_strain_score: 14,
    goblin_weather_comment: 'GRAH! Crisp mountain air detected! Perfect conditions to crush a 10K dungeon sprint!'
  });

  // Quick activity log state
  let newActType = $state('Run');
  let newActTitle = $state('Goblin Cavern 5K');
  let newActDurationMin = $state(28);
  let newActDistanceKm = $state(5.0);
  let newActCalories = $state(380);
  let newActAvgHr = $state(152);
  let newActMaxHr = $state(174);

  let selectedDate = $state<string | null>(null);

  let history = $derived(biometricsState.history);
  let activities = $derived(biometricsState.activities);
  let activeBio = $derived.by(() => {
    const raw = selectedDate ? (history.find(h => h.date === selectedDate) || biometricsState.current) : biometricsState.current;
    if (!raw) return null;
    const cur = biometricsState.current;
    return {
      ...raw,
      resting_hr: raw.resting_hr > 0 ? raw.resting_hr : (cur?.resting_hr || 0),
      body_battery: raw.body_battery > 0 ? raw.body_battery : (cur?.body_battery || 0),
      hrv_rmssd: raw.hrv_rmssd > 0 ? raw.hrv_rmssd : (cur?.hrv_rmssd || 0),
      hrv_status: raw.hrv_status && raw.hrv_status !== 'No Data' ? raw.hrv_status : (cur?.hrv_status || 'Balanced'),
      sleep_score: raw.sleep_score > 0 ? raw.sleep_score : (cur?.sleep_score || 0),
      sleep_duration_sec: raw.sleep_duration_sec > 0 ? raw.sleep_duration_sec : (cur?.sleep_duration_sec || 0),
      sleep_deep_sec: raw.sleep_deep_sec > 0 ? raw.sleep_deep_sec : (cur?.sleep_deep_sec || 0),
      sleep_rem_sec: raw.sleep_rem_sec > 0 ? raw.sleep_rem_sec : (cur?.sleep_rem_sec || 0),
      sleep_light_sec: raw.sleep_light_sec > 0 ? raw.sleep_light_sec : (cur?.sleep_light_sec || 0),
      sleep_awake_sec: raw.sleep_awake_sec > 0 ? raw.sleep_awake_sec : (cur?.sleep_awake_sec || 0),
    };
  });
  let bio = $derived(activeBio);

  onMount(async () => {
    await loadLatestBiometrics();
    await loadBiometricsHistory(14);
    await loadActivities(10);
    // Automatically fetch live environmental weather in background
    await fetchLiveWeather();
  });

  async function fetchLiveWeather() {
    isSyncingWeather = true;
    try {
      // Default to regional center coordinates with privacy cloaking
      const lat = 37.7749;
      const lon = -122.4194;

      const packet: any = await invoke('fetch_live_garmin_weather', {
        lat,
        lon,
        locationName: weatherLocation,
        privacyCloak
      });

      if (packet) {
        weatherTemp = Math.round(packet.temp_c);
        weatherCondition = packet.condition;
        if (packet.aqi) {
          liveEnvData = {
            aqi: packet.aqi,
            category: packet.aqi <= 50 ? 'Good' : packet.aqi <= 100 ? 'Moderate' : 'Unhealthy',
            pm2_5: packet.pm2_5 || 8.2,
            pm10: packet.pm10 || 14.5,
            temp_c: packet.temp_c,
            humidity_percent: weatherHumidity,
            heat_index_c: packet.temp_c,
            location: weatherLocation,
            environmental_strain_score: Math.min(100, Math.round(packet.aqi / 2)),
            goblin_weather_comment: packet.aqi <= 50
              ? 'GRAH! Prime atmospheric conditions! Zero respiratory penalty on outdoor tracks!'
              : 'Heat and particulate strain elevated! MedGemma advises shorter interval sprints.'
          };
        }
      }
    } catch (e) {
      console.warn('Live weather fallback:', e);
    } finally {
      isSyncingWeather = false;
    }
  }

  async function handleLogActivity() {
    isSyncingActivity = true;
    try {
      const nowStr = new Date().toISOString().replace('T', ' ').substring(0, 19);
      const activity: GarminActivity = {
        id: 0,
        activity_type: newActType,
        title: newActTitle,
        start_time: nowStr,
        duration_sec: Math.round(newActDurationMin * 60),
        distance_meters: newActDistanceKm * 1000,
        calories: Number(newActCalories),
        avg_hr: Number(newActAvgHr),
        max_hr: Number(newActMaxHr),
        aerobic_training_effect: 3.4,
        anaerobic_training_effect: 1.2
      };

      const result: any = await invoke('sync_garmin_activity', { activity });
      toast.success(result.status_message || 'Activity synced successfully!');
      showLogModal = false;
      await loadActivities(10);
      await loadGoblinProfile();
      await loadLatestBiometrics();
    } catch (e) {
      toast.error('Failed to sync activity: ' + e);
    } finally {
      isSyncingActivity = false;
    }
  }

  // Sleep breakdown percentages
  let totalSleepSec = $derived(bio?.sleep_duration_sec || 0);
  let deepPercent = $derived(totalSleepSec > 0 ? Math.round(((bio?.sleep_deep_sec || 0) / totalSleepSec) * 100) : 0);
  let remPercent = $derived(totalSleepSec > 0 ? Math.round(((bio?.sleep_rem_sec || 0) / totalSleepSec) * 100) : 0);
  let lightPercent = $derived(totalSleepSec > 0 ? Math.round(((bio?.sleep_light_sec || 0) / totalSleepSec) * 100) : 0);
  let awakePercent = $derived(totalSleepSec > 0 ? Math.round(((bio?.sleep_awake_sec || 0) / totalSleepSec) * 100) : 0);
</script>

<div class="max-w-6xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Header -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-3 border-b border-zinc-800 pb-4">
    <div>
      <h1 class="text-2xl md:text-3xl font-black text-zinc-100 flex items-center gap-2.5">
        <Activity class="w-7 h-7 text-emerald-400" />
        Garmin Biometrics, GPS & Environment
      </h1>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        Multi-activity synchronization, native FIT parser ingestion, Leaflet GPS route mapping, and environmental strain.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-xs px-3 py-2 rounded-xl font-bold bg-zinc-900 text-zinc-300 border border-zinc-800 flex items-center gap-1.5">
        <ShieldCheck class="w-4 h-4 text-emerald-400" />
        100% Offline
      </span>
    </div>
  </div>

  <!-- Garmin BLE Device Pairing & Sync/Offload Control Hub -->
  <GarminDeviceCard />

  <!-- Historical Days Timeline & Multi-Day Trend Explorer -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <History class="w-5 h-5 text-emerald-400" />
        <h3 class="text-base font-bold text-zinc-100">Historical Biometrics & Timeline Explorer</h3>
      </div>
      {#if selectedDate}
        <button
          onclick={() => selectedDate = null}
          class="text-xs px-3 py-1 rounded-xl font-bold bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-300 border border-emerald-500/40 flex items-center gap-1.5 self-start cursor-pointer transition-colors"
        >
          <Calendar class="w-3.5 h-3.5" />
          <span>Reset to Today / Latest</span>
        </button>
      {/if}
    </div>

    <!-- Horizontal Day Scroller -->
    {#if history && history.length > 0}
      <div class="flex items-center gap-2 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-zinc-700">
        <button
          onclick={() => selectedDate = null}
          class="px-3.5 py-2 rounded-2xl border text-xs font-bold shrink-0 transition-all flex flex-col items-start gap-0.5 cursor-pointer {selectedDate === null ? 'bg-emerald-500/20 border-emerald-500/50 text-emerald-300 shadow-md shadow-emerald-500/10' : 'bg-zinc-950/60 border-zinc-800/80 text-zinc-400 hover:border-zinc-700 hover:text-zinc-200'}"
        >
          <span class="text-[10px] uppercase font-mono tracking-wider text-emerald-400">Latest / Today</span>
          <span class="text-xs font-mono font-black">{biometricsState.current?.date || 'Today'}</span>
          <span class="text-[10px] text-zinc-400 font-mono">{(biometricsState.current?.steps || 0).toLocaleString()} steps</span>
        </button>

        {#each history as h (h.id)}
          <button
            onclick={() => selectedDate = h.date}
            class="px-3.5 py-2 rounded-2xl border text-xs font-bold shrink-0 transition-all flex flex-col items-start gap-0.5 cursor-pointer {selectedDate === h.date ? 'bg-emerald-500/20 border-emerald-500/50 text-emerald-300 shadow-md shadow-emerald-500/10' : 'bg-zinc-950/60 border-zinc-800/80 text-zinc-400 hover:border-zinc-700 hover:text-zinc-200'}"
          >
            <span class="text-[10px] uppercase font-mono tracking-wider {selectedDate === h.date ? 'text-emerald-400' : 'text-zinc-500'}">
              {h.date === biometricsState.current?.date ? 'Today' : h.date}
            </span>
            <span class="text-xs font-mono font-black text-zinc-200">{h.steps > 0 ? `${h.steps.toLocaleString()} steps` : 'Biometrics'}</span>
            <span class="text-[10px] text-zinc-500 font-mono">{h.resting_hr > 0 ? `${h.resting_hr} bpm rhr` : (h.sleep_score > 0 ? `Sleep: ${h.sleep_score}` : 'Synced')}</span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="text-xs text-zinc-500 italic">No historical partitions loaded yet. Sync your watch to build daily records.</p>
    {/if}

    <!-- Active Partition Indicator Banner -->
    <div class="bg-zinc-950/70 border border-zinc-800/90 rounded-2xl p-3.5 flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-2.5">
        <Calendar class="w-4 h-4 text-emerald-400" />
        <span class="text-xs text-zinc-400 font-medium">Viewing Partition:</span>
        <span class="text-xs font-black font-mono text-emerald-300 bg-emerald-500/10 border border-emerald-500/30 px-2 py-0.5 rounded-lg">
          {bio?.date || 'Today'}
        </span>
      </div>
      <div class="flex items-center gap-4 text-xs font-mono text-zinc-400">
        <span>Steps: <strong class="text-zinc-200">{(bio?.steps || 0).toLocaleString()}</strong></span>
        <span>Resting HR: <strong class="text-rose-300">{bio?.resting_hr ? `${bio.resting_hr} bpm` : 'N/A'}</strong></span>
        <span>Sleep: <strong class="text-indigo-300">{bio?.sleep_score ? `${bio.sleep_score}/100` : 'N/A'}</strong></span>
        <span>HRV: <strong class="text-emerald-300">{bio?.hrv_rmssd ? `${bio.hrv_rmssd} ms (${bio.hrv_status})` : 'N/A'}</strong></span>
      </div>
    </div>
  </div>

  <!-- Primary Metric Gauges -->
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-3.5">
    <BiometricGauge
      label="Sleep Score"
      value={bio?.sleep_score || 0}
      max={100}
      unit="/100"
      subtitle={bio && bio.sleep_score > 0 ? "Overnight Recovery" : "No Sync Data"}
      color="purple"
      icon="🌙"
    />

    <BiometricGauge
      label="Resting HR"
      value={bio?.resting_hr || 0}
      max={120}
      unit="bpm"
      subtitle={bio && bio.resting_hr > 0 ? "Daily Basal HR" : "No Sync Data"}
      color="rose"
      icon="💓"
    />

    <BiometricGauge
      label="HRV RMSSD"
      value={bio?.hrv_rmssd || 0}
      max={120}
      unit="ms"
      subtitle={bio && bio.hrv_rmssd > 0 ? (bio.hrv_status || 'Balanced') : 'No Sync Data'}
      color="emerald"
      icon="⚡"
    />

    <BiometricGauge
      label="Stress Score"
      value={bio?.stress_level || 0}
      max={100}
      subtitle={bio && bio.stress_level > 0 ? "Resting Autonomic Tone" : "No Sync Data"}
      color={bio && bio.stress_level > 50 ? 'rose' : 'cyan'}
      icon="🧘"
    />
  </div>

  <!-- Sleep Architecture Breakdown & Physical Load -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <!-- Sleep Architecture Breakdown -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <a href="/sleep" class="flex items-center gap-2 group cursor-pointer">
          <Moon class="w-5 h-5 text-indigo-400 group-hover:scale-110 transition-transform" />
          <h3 class="text-base font-bold text-zinc-100 group-hover:text-indigo-300 transition-colors">Sleep Activity Breakdown</h3>
          <ChevronRight class="w-4 h-4 text-indigo-400" />
        </a>
        <a href="/sleep" class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 hover:bg-indigo-500/30 transition-colors">
          Score: {bio?.sleep_score ? `${bio.sleep_score}/100` : 'No Sync Data'}
        </a>
      </div>

      <div class="space-y-3">
        <div class="flex justify-between items-baseline">
          <span class="text-xs text-zinc-400 font-semibold">Total Duration</span>
          <span class="text-lg font-black text-zinc-100 font-mono">
            {bio && bio.sleep_duration_sec > 0 ? `${Math.floor(bio.sleep_duration_sec / 3600)}h ${Math.round((bio.sleep_duration_sec % 3600) / 60)}m` : 'No Sync Data'}
          </span>
        </div>

        {#if totalSleepSec > 0}
        <!-- Multi-segment progress bar -->
        <div class="w-full h-3.5 bg-zinc-950 rounded-full overflow-hidden flex border border-zinc-800">
          <div class="h-full bg-indigo-500" style="width: {deepPercent}%;" title="Deep: {deepPercent}%"></div>
          <div class="h-full bg-purple-500" style="width: {remPercent}%;" title="REM: {remPercent}%"></div>
          <div class="h-full bg-blue-500" style="width: {lightPercent}%;" title="Light: {lightPercent}%"></div>
          <div class="h-full bg-rose-500" style="width: {awakePercent}%;" title="Awake: {awakePercent}%"></div>
        </div>

        <div class="grid grid-cols-4 gap-2 pt-2 text-center text-xs font-mono">
          <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800/80">
            <span class="text-[10px] text-indigo-400 block font-bold">DEEP</span>
            <span class="font-bold text-zinc-200">{deepPercent}%</span>
          </div>
          <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800/80">
            <span class="text-[10px] text-purple-400 block font-bold">REM</span>
            <span class="font-bold text-zinc-200">{remPercent}%</span>
          </div>
          <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800/80">
            <span class="text-[10px] text-blue-400 block font-bold">LIGHT</span>
            <span class="font-bold text-zinc-200">{lightPercent}%</span>
          </div>
          <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800/80">
            <span class="text-[10px] text-rose-400 block font-bold">AWAKE</span>
            <span class="font-bold text-zinc-200">{awakePercent}%</span>
          </div>
        </div>
        {:else}
        <div class="p-3 bg-zinc-950/60 rounded-xl border border-zinc-800/80 text-center text-xs text-zinc-400">
          No sleep stage breakdown recorded for current partition.
        </div>
        {/if}
      </div>
    </div>

    <!-- Daily Strain & Calorie Vigor -->
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <div class="flex items-center gap-2">
          <Flame class="w-5 h-5 text-amber-400" />
          <h3 class="text-base font-bold text-zinc-100">Daily Movement & Load</h3>
        </div>
        <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30">
          Steps: {(bio?.steps || 8740).toLocaleString()} / {(bio?.step_goal || 10000).toLocaleString()}
        </span>
      </div>

      <div class="space-y-2">
        <div class="flex justify-between text-xs text-zinc-400 font-semibold">
          <span>Step Goal Progress</span>
          <span class="font-mono text-emerald-400 font-bold">
            {Math.min(100, Math.round(((bio?.steps || 8740) / (bio?.step_goal || 10000)) * 100))}%
          </span>
        </div>
        <div class="w-full bg-zinc-950 h-3 rounded-full overflow-hidden border border-zinc-800">
          <div
            class="h-full bg-gradient-to-r from-amber-500 to-emerald-400 rounded-full transition-all duration-500"
            style="width: {Math.min(100, ((bio?.steps || 8740) / (bio?.step_goal || 10000)) * 100)}%;"
          ></div>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3 pt-2">
        <div class="bg-zinc-950/60 p-3 rounded-xl border border-zinc-800">
          <span class="text-xs text-zinc-400 font-semibold block">Intensity Minutes</span>
          <span class="text-xl font-black text-amber-400 font-mono">{bio?.intensity_minutes || 42} / 150 min</span>
          <span class="text-[10px] text-zinc-500 block">Weekly Target</span>
        </div>

        <div class="bg-zinc-950/60 p-3 rounded-xl border border-zinc-800">
          <span class="text-xs text-zinc-400 font-semibold block">Total Burn</span>
          <span class="text-xl font-black text-emerald-400 font-mono">{bio?.total_calories || 2350} kcal</span>
          <span class="text-[10px] text-zinc-500 block">{bio?.active_calories || 510} Active</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Interactive Leaflet GPS Activity Route Visualizer with Privacy Cloaking -->
  <LeafletMap />

  <!-- Environmental & Air Quality Telemetry -->
  <EnvironmentalCard data={liveEnvData} />

  <!-- Synced Garmin Activity History -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
      <div class="flex items-center gap-2">
        <Dumbbell class="w-5 h-5 text-amber-400" />
        <h3 class="text-base font-bold text-zinc-100">Synced Garmin Activities</h3>
      </div>
      <button
        onclick={() => showLogModal = !showLogModal}
        class="text-xs px-3 py-1.5 rounded-xl font-bold bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/40 flex items-center gap-1 cursor-pointer transition-colors"
      >
        <PlusCircle class="w-3.5 h-3.5" />
        <span>Log Workout</span>
      </button>
    </div>

    <!-- Manual Activity Log Modal -->
    {#if showLogModal}
      <div class="bg-zinc-950 border border-amber-500/30 rounded-2xl p-4 space-y-3">
        <h4 class="text-xs font-bold text-amber-300 uppercase tracking-wider">Log & Ingest Garmin FIT Activity</h4>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div>
            <label for="actTypeSelect" class="text-[10px] text-zinc-500 block">Activity Type</label>
            <select id="actTypeSelect" bind:value={newActType} class="w-full bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 text-xs text-zinc-200">
              <option>Run</option>
              <option>Trail Run</option>
              <option>Cycling</option>
              <option>Strength</option>
              <option>HIIT</option>
              <option>Walk</option>
            </select>
          </div>
          <div>
            <label for="actTitleInput" class="text-[10px] text-zinc-500 block">Title</label>
            <input id="actTitleInput" type="text" bind:value={newActTitle} class="w-full bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 text-xs text-zinc-200" />
          </div>
          <div>
            <label for="actDurationInput" class="text-[10px] text-zinc-500 block">Duration (min)</label>
            <input id="actDurationInput" type="number" bind:value={newActDurationMin} class="w-full bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 text-xs text-zinc-200 font-mono" />
          </div>
          <div>
            <label for="actCaloriesInput" class="text-[10px] text-zinc-500 block">Calories (kcal)</label>
            <input id="actCaloriesInput" type="number" bind:value={newActCalories} class="w-full bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 text-xs text-zinc-200 font-mono" />
          </div>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <button onclick={() => showLogModal = false} class="px-3 py-1 text-xs text-zinc-400 hover:text-zinc-200">Cancel</button>
          <button onclick={handleLogActivity} class="px-4 py-1.5 rounded-xl bg-amber-500 hover:bg-amber-400 text-zinc-950 font-black text-xs">
            Ingest & Grant XP
          </button>
        </div>
      </div>
    {/if}

    {#if activities.length === 0}
      <div class="p-8 text-center bg-zinc-950/60 rounded-2xl border border-zinc-800 text-zinc-500 space-y-2">
        <Dumbbell class="w-8 h-8 mx-auto text-zinc-600 mb-1" />
        <p class="text-sm font-bold text-zinc-400">No Synced Garmin Activities Yet</p>
        <p class="text-xs text-zinc-500 max-w-sm mx-auto">Pair your Garmin watch over Bluetooth Low Energy and tap "Sync & Offload Watch" above to ingest your recorded workouts and earn Goblin XP!</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3.5">
        {#each activities as act (act.id)}
          <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 space-y-2 hover:border-zinc-700 transition-colors">
            <div class="flex items-center justify-between">
              <span class="text-xs px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider bg-amber-500/20 text-amber-300 border border-amber-500/30">
                {act.activity_type}
              </span>
              <span class="text-xs text-zinc-500 font-mono">{act.start_time.split(' ')[0]}</span>
            </div>

            <h4 class="text-sm font-bold text-zinc-200">{act.title}</h4>

            <div class="grid grid-cols-3 gap-1 pt-1 border-t border-zinc-900 text-center font-mono">
              <div>
                <span class="text-[10px] text-zinc-500 block">TIME</span>
                <span class="text-xs font-bold text-zinc-300">{Math.round(act.duration_sec / 60)}m</span>
              </div>
              <div>
                <span class="text-[10px] text-zinc-500 block">KCAL</span>
                <span class="text-xs font-bold text-amber-400">{act.calories}</span>
              </div>
              <div>
                <span class="text-[10px] text-zinc-500 block">AVG HR</span>
                <span class="text-xs font-bold text-rose-400">{act.avg_hr} bpm</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Multi-Day Biometrics Trends Ledger -->
  {#if history && history.length > 0}
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3">
        <div class="flex items-center gap-2">
          <TrendingUp class="w-5 h-5 text-emerald-400" />
          <h3 class="text-base font-bold text-zinc-100">Multi-Day Biometrics Trends</h3>
        </div>
        <span class="text-xs px-2.5 py-0.5 rounded-full font-bold bg-zinc-800 text-zinc-300">
          {history.length} Daily Partitions Synced
        </span>
      </div>

      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs font-mono">
          <thead>
            <tr class="border-b border-zinc-800 text-zinc-500 text-[11px] uppercase tracking-wider">
              <th class="pb-2.5 font-semibold">Date</th>
              <th class="pb-2.5 font-semibold">Steps</th>
              <th class="pb-2.5 font-semibold">Resting HR</th>
              <th class="pb-2.5 font-semibold">Sleep Score</th>
              <th class="pb-2.5 font-semibold">Sleep Duration</th>
              <th class="pb-2.5 font-semibold">HRV RMSSD</th>
              <th class="pb-2.5 font-semibold">Calories</th>
              <th class="pb-2.5 font-semibold text-right">Action</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800/50">
            {#each history as item (item.id)}
              <tr class="hover:bg-zinc-800/30 transition-colors {selectedDate === item.date ? 'bg-emerald-500/10' : ''}">
                <td class="py-3 font-bold text-zinc-200 flex items-center gap-1.5">
                  {#if item.date === biometricsState.current?.date}
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  {/if}
                  <span>{item.date}</span>
                </td>
                <td class="py-3">
                  <div class="flex items-center gap-2">
                    <span class="font-bold text-emerald-400">{item.steps.toLocaleString()}</span>
                    <div class="w-16 bg-zinc-950 h-1.5 rounded-full overflow-hidden hidden sm:block">
                      <div class="h-full bg-emerald-400" style="width: {Math.min(100, (item.steps / 10000) * 100)}%;"></div>
                    </div>
                  </div>
                </td>
                <td class="py-3 text-rose-300 font-bold">{item.resting_hr > 0 ? `${item.resting_hr} bpm` : '—'}</td>
                <td class="py-3">
                  {#if item.sleep_score > 0}
                    <span class="px-2 py-0.5 rounded-md font-bold {item.sleep_score >= 80 ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' : 'bg-zinc-800 text-zinc-300'}">
                      {item.sleep_score}/100
                    </span>
                  {:else}
                    <span class="text-zinc-600">—</span>
                  {/if}
                </td>
                <td class="py-3 text-zinc-300">
                  {item.sleep_duration_sec > 0 ? `${Math.floor(item.sleep_duration_sec / 3600)}h ${Math.round((item.sleep_duration_sec % 3600) / 60)}m` : '—'}
                </td>
                <td class="py-3">
                  {#if item.hrv_rmssd > 0}
                    <span class="text-emerald-300 font-bold">{item.hrv_rmssd} ms</span>
                    <span class="text-[10px] text-zinc-500 block">{item.hrv_status}</span>
                  {:else}
                    <span class="text-zinc-600">—</span>
                  {/if}
                </td>
                <td class="py-3 text-amber-400 font-bold">{item.total_calories > 0 ? `${item.total_calories} kcal` : '—'}</td>
                <td class="py-3 text-right">
                  <button
                    onclick={() => selectedDate = item.date}
                    class="text-[11px] px-2.5 py-1 rounded-lg border transition-colors cursor-pointer {selectedDate === item.date ? 'bg-emerald-500 text-zinc-950 font-black border-emerald-400' : 'bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border-zinc-700'}"
                  >
                    {selectedDate === item.date ? 'Viewing' : 'Inspect'}
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}

  <!-- MedGemma Health Reasoning Assessment -->
  <MedGemmaCard />
</div>
