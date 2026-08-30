<script lang="ts">
  import type { AirQualityData } from '$lib/types/garmin';
  import { Wind, Thermometer, Droplets, Sun, AlertTriangle, ShieldCheck, Sparkles, CloudRain } from 'lucide-svelte';

  interface Props {
    data?: Partial<AirQualityData>;
  }

  let { data = {} }: Props = $props();

  const envData: AirQualityData = $derived({
    aqi: data.aqi ?? 34,
    category: data.category ?? 'Good',
    pm2_5: data.pm2_5 ?? 8.2,
    pm10: data.pm10 ?? 14.5,
    o3: data.o3 ?? 28.0,
    no2: data.no2 ?? 12.1,
    temp_c: data.temp_c ?? 19.5,
    humidity_percent: data.humidity_percent ?? 54,
    heat_index_c: data.heat_index_c ?? 19.8,
    location: data.location ?? 'Goblin Mountain Cavern & Trails',
    environmental_strain_score: data.environmental_strain_score ?? 12,
    goblin_weather_comment: data.goblin_weather_comment ?? 'GRAH! Crisp mountain air detected! Perfect conditions to crush a 10K dungeon sprint!'
  });

  function getAqiColor(aqi: number): { bg: string; text: string; border: string } {
    if (aqi <= 50) return { bg: 'bg-emerald-500/20', text: 'text-emerald-400', border: 'border-emerald-500/40' };
    if (aqi <= 100) return { bg: 'bg-amber-500/20', text: 'text-amber-400', border: 'border-amber-500/40' };
    if (aqi <= 150) return { bg: 'bg-orange-500/20', text: 'text-orange-400', border: 'border-orange-500/40' };
    return { bg: 'bg-rose-500/20', text: 'text-rose-400', border: 'border-rose-500/40' };
  }

  let aqiTheme = $derived(getAqiColor(envData.aqi));
</script>

<div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-emerald-950/30 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-3">
    <div class="flex items-center gap-2.5">
      <div class="w-9 h-9 rounded-xl bg-cyan-500/20 text-cyan-400 border border-cyan-500/40 flex items-center justify-center">
        <Wind class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-base font-bold text-zinc-100 flex items-center gap-2">
          Environmental Strain & Air Quality
          <span class="text-[10px] px-2 py-0.5 rounded-full font-extrabold {aqiTheme.bg} {aqiTheme.text} border {aqiTheme.border} uppercase tracking-wider">
            AQI {envData.aqi} • {envData.category}
          </span>
        </h3>
        <span class="text-xs text-zinc-400">{envData.location}</span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-xs px-2.5 py-1 rounded-xl bg-zinc-950 text-zinc-300 border border-zinc-800 flex items-center gap-1">
        <Thermometer class="w-3.5 h-3.5 text-amber-400" />
        <span class="font-mono font-bold text-amber-400">{envData.temp_c}°C</span>
        <span class="text-zinc-500">Feels {envData.heat_index_c}°C</span>
      </span>
    </div>
  </div>

  <!-- Telemetry Metrics Grid -->
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
    <!-- PM2.5 Fine Particulates -->
    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3 space-y-1">
      <span class="text-[10px] font-bold text-zinc-500 uppercase">PM2.5 Particles</span>
      <div class="flex items-baseline justify-between">
        <span class="text-base font-black text-zinc-200 font-mono">{envData.pm2_5}</span>
        <span class="text-[10px] text-zinc-500 font-mono">µg/m³</span>
      </div>
      <div class="w-full bg-zinc-900 h-1.5 rounded-full overflow-hidden">
        <div class="h-full bg-emerald-400 rounded-full" style="width: {Math.min(100, (envData.pm2_5 / 35) * 100)}%;"></div>
      </div>
    </div>

    <!-- PM10 Coarse Particulates -->
    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3 space-y-1">
      <span class="text-[10px] font-bold text-zinc-500 uppercase">PM10 Coarse</span>
      <div class="flex items-baseline justify-between">
        <span class="text-base font-black text-zinc-200 font-mono">{envData.pm10}</span>
        <span class="text-[10px] text-zinc-500 font-mono">µg/m³</span>
      </div>
      <div class="w-full bg-zinc-900 h-1.5 rounded-full overflow-hidden">
        <div class="h-full bg-emerald-400 rounded-full" style="width: {Math.min(100, (envData.pm10 / 50) * 100)}%;"></div>
      </div>
    </div>

    <!-- Relative Humidity -->
    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3 space-y-1">
      <span class="text-[10px] font-bold text-zinc-500 uppercase">Humidity</span>
      <div class="flex items-baseline justify-between">
        <span class="text-base font-black text-cyan-400 font-mono">{envData.humidity_percent}%</span>
        <Droplets class="w-3.5 h-3.5 text-cyan-500" />
      </div>
      <div class="w-full bg-zinc-900 h-1.5 rounded-full overflow-hidden">
        <div class="h-full bg-cyan-400 rounded-full" style="width: {envData.humidity_percent}%;"></div>
      </div>
    </div>

    <!-- Environmental Strain Score -->
    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3 space-y-1">
      <span class="text-[10px] font-bold text-zinc-500 uppercase">Thermal Strain</span>
      <div class="flex items-baseline justify-between">
        <span class="text-base font-black text-emerald-400 font-mono">{envData.environmental_strain_score}/100</span>
        <span class="text-[10px] text-emerald-400/80 font-bold">Low</span>
      </div>
      <div class="w-full bg-zinc-900 h-1.5 rounded-full overflow-hidden">
        <div class="h-full bg-emerald-400 rounded-full" style="width: {envData.environmental_strain_score}%;"></div>
      </div>
    </div>
  </div>

  <!-- Goblin Environmental Coach Advice -->
  <div class="bg-zinc-950/90 border border-emerald-500/20 rounded-2xl p-3.5 flex items-start gap-3">
    <div class="w-7 h-7 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center shrink-0 mt-0.5">
      <Sparkles class="w-4 h-4" />
    </div>
    <div class="space-y-0.5">
      <span class="text-[11px] font-bold text-emerald-400 uppercase tracking-wider">Goblin Environmental Telemetry</span>
      <p class="text-xs text-zinc-300 italic leading-relaxed">
        "{envData.goblin_weather_comment}"
      </p>
    </div>
  </div>
</div>
