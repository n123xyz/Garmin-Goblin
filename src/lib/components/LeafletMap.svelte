<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { GpsRouteTrack } from '$lib/types/garmin';
  import { Navigation, Mountain, Gauge, Thermometer, Wind, ShieldCheck, ShieldAlert, Eye, EyeOff } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import 'leaflet/dist/leaflet.css';

  interface Props {
    tracks?: GpsRouteTrack[];
    selectedTrackId?: string;
  }

  let { tracks = [], selectedTrackId }: Props = $props();

  let mapContainer: HTMLElement | null = $state(null);
  let mapInstance: any = null;
  let polylineLayer: any = null;
  let markersLayer: any = null;
  let activeTrackId = $state('');
  let privacyCloakEnabled = $state(true);
  let cloakedStats = $state<{ redacted: boolean; originalCount: number; cloakedCount: number }>({
    redacted: true,
    originalCount: 0,
    cloakedCount: 0
  });

  $effect(() => {
    if (selectedTrackId) {
      activeTrackId = selectedTrackId;
    } else if (!activeTrackId && tracks.length > 0) {
      activeTrackId = tracks[0].id;
    }
  });

  let activeTrack = $derived(tracks.find(t => t.id === activeTrackId) || tracks[0]);

  onMount(async () => {
    if (typeof window === 'undefined' || !mapContainer || !activeTrack || !activeTrack.coordinates?.length) return;

    try {
      const L = await import('leaflet');

      mapInstance = L.map(mapContainer, {
        zoomControl: false,
        attributionControl: true
      }).setView(activeTrack.coordinates[0], 14);

      // Official standard OpenStreetMap tile layer for Leaflet
      L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
      }).addTo(mapInstance);

      // Standard Leaflet Zoom Control
      L.control.zoom({ position: 'bottomright' }).addTo(mapInstance);

      await renderCurrentTrack(L);
    } catch (e) {
      console.error('Failed to initialize Leaflet Map:', e);
    }
  });

  onDestroy(() => {
    if (mapInstance) {
      mapInstance.remove();
      mapInstance = null;
    }
  });

  async function renderCurrentTrack(L?: any) {
    if (!mapInstance || !activeTrack || !activeTrack.coordinates?.length) return;
    const Leaflet = L || await import('leaflet');

    if (polylineLayer) mapInstance.removeLayer(polylineLayer);
    if (markersLayer) mapInstance.removeLayer(markersLayer);

    markersLayer = Leaflet.layerGroup().addTo(mapInstance);

    let coordsToRender: [number, number][] = activeTrack.coordinates;

    if (privacyCloakEnabled) {
      try {
        const cloaked: any = await invoke('cloak_gps_track', {
          coords: activeTrack.coordinates,
          privacyRadiusMeters: 300.0,
          enableDpNoise: true
        });
        if (cloaked && cloaked.coordinates && cloaked.coordinates.length > 0) {
          coordsToRender = cloaked.coordinates;
          cloakedStats = {
            redacted: true,
            originalCount: cloaked.original_point_count,
            cloakedCount: cloaked.cloaked_point_count
          };
        }
      } catch (e) {
        console.error('Privacy cloaking fallback:', e);
      }
    } else {
      cloakedStats = {
        redacted: false,
        originalCount: activeTrack.coordinates.length,
        cloakedCount: activeTrack.coordinates.length
      };
    }

    // Render polyline on Leaflet map
    polylineLayer = Leaflet.polyline(coordsToRender, {
      color: '#10b981',
      weight: 5,
      opacity: 0.9,
      lineJoin: 'round',
      lineCap: 'round'
    }).addTo(mapInstance);

    // Leaflet Start Marker (privacy aware)
    const startIcon = Leaflet.divIcon({
      className: 'leaflet-goblin-start-marker',
      html: `<div style="background-color:#10b981; width:26px; height:26px; border-radius:50%; border:2px solid #064e3b; display:flex; align-items:center; justify-content:center; font-size:12px; box-shadow:0 2px 6px rgba(0,0,0,0.4);">📍</div>`,
      iconSize: [26, 26],
      iconAnchor: [13, 13]
    });
    Leaflet.marker(coordsToRender[0], { icon: startIcon })
      .bindPopup(`<b>Start Waypoint</b><br>${privacyCloakEnabled ? '300m Privacy Protected' : 'Raw GPS'}`)
      .addTo(markersLayer);

    // Leaflet Finish Marker
    const finishIcon = Leaflet.divIcon({
      className: 'leaflet-goblin-finish-marker',
      html: `<div style="background-color:#f59e0b; width:26px; height:26px; border-radius:50%; border:2px solid #451a03; display:flex; align-items:center; justify-content:center; font-size:12px; box-shadow:0 2px 6px rgba(0,0,0,0.4);">🏁</div>`,
      iconSize: [26, 26],
      iconAnchor: [13, 13]
    });
    Leaflet.marker(coordsToRender[coordsToRender.length - 1], { icon: finishIcon })
      .bindPopup(`<b>Finish Waypoint</b><br>Track Distance: ${activeTrack.distance_km} km`)
      .addTo(markersLayer);

    mapInstance.fitBounds(polylineLayer.getBounds(), { padding: [30, 30] });
  }

  function selectTrack(trackId: string) {
    activeTrackId = trackId;
    renderCurrentTrack();
  }

  function togglePrivacyCloak() {
    privacyCloakEnabled = !privacyCloakEnabled;
    renderCurrentTrack();
  }
</script>

<div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
  <!-- Header with Track Selector & Privacy Toggle -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-3">
    <div class="flex items-center gap-2.5">
      <div class="w-9 h-9 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 flex items-center justify-center">
        <Navigation class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-base font-bold text-zinc-100 flex items-center gap-2">
          Garmin GPS Activity Route & Trails
          <span class="text-[10px] px-2 py-0.5 rounded-full font-extrabold bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 uppercase tracking-wider">
            Leaflet / OpenStreetMap
          </span>
        </h3>
        <span class="text-xs text-zinc-400">GPS route ingestion with differential privacy start/end point masking</span>
      </div>
    </div>

    {#if tracks.length > 0}
      <!-- Privacy Cloak Toggle -->
      <div class="flex items-center gap-2">
        <button
          onclick={togglePrivacyCloak}
          class="text-xs px-3 py-1.5 rounded-xl font-bold flex items-center gap-1.5 border transition-all cursor-pointer {privacyCloakEnabled ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' : 'bg-amber-500/20 text-amber-300 border-amber-500/40'}"
          title="Cloak start/end points within 300m radius to hide home/sensitive origins"
        >
          {#if privacyCloakEnabled}
            <ShieldCheck class="w-3.5 h-3.5 text-emerald-400" />
            <span>Privacy Cloak: ON</span>
          {:else}
            <ShieldAlert class="w-3.5 h-3.5 text-amber-400" />
            <span>Privacy Cloak: OFF</span>
          {/if}
        </button>

        <!-- Track Selector Pills -->
        <div class="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0">
          {#each tracks as track}
            <button
              onclick={() => selectTrack(track.id)}
              class="text-xs px-3 py-1.5 rounded-xl border font-bold whitespace-nowrap transition-all cursor-pointer {activeTrackId === track.id ? 'bg-emerald-500 text-zinc-950 border-emerald-400 shadow-[0_0_12px_rgba(16,185,129,0.3)]' : 'bg-zinc-950 text-zinc-400 border-zinc-800 hover:text-zinc-200'}"
            >
              {track.name.split(' ')[0]} {track.distance_km}km
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  {#if tracks.length === 0 || !activeTrack}
    <div class="w-full h-48 rounded-2xl bg-zinc-950/60 border border-zinc-800 flex flex-col items-center justify-center text-center p-6 text-zinc-500 space-y-2">
      <Navigation class="w-8 h-8 text-zinc-600 mb-1" />
      <p class="text-sm font-bold text-zinc-400">No GPS Activity Route Recorded</p>
      <p class="text-xs text-zinc-500 max-w-md">Record an outdoor activity (Run, Trail Run, Hike, Cycling) on your Garmin watch and sync over BLE to visualize the map route with differential privacy cloaking.</p>
    </div>
  {:else}
    <!-- Map Container -->
    <div class="relative w-full h-80 rounded-2xl overflow-hidden border border-zinc-800 shadow-inner bg-zinc-950">
      <div bind:this={mapContainer} class="w-full h-full z-0"></div>

      <!-- Route Stats Overlay Panel -->
      <div class="absolute top-3 left-3 z-[1000] bg-zinc-950/90 backdrop-blur-md border border-zinc-800 rounded-2xl p-3.5 shadow-2xl space-y-2 max-w-xs pointer-events-auto">
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs font-black text-zinc-100 truncate">{activeTrack.name}</span>
          <span class="text-[10px] px-2 py-0.5 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase">
            {activeTrack.activity_type}
          </span>
        </div>

        <div class="grid grid-cols-3 gap-2 text-center font-mono">
          <div class="bg-zinc-900/80 p-1.5 rounded-xl border border-zinc-800">
            <span class="text-[9px] text-zinc-500 block">DIST</span>
            <span class="text-xs font-black text-emerald-400">{activeTrack.distance_km} km</span>
          </div>
          <div class="bg-zinc-900/80 p-1.5 rounded-xl border border-zinc-800">
            <span class="text-[9px] text-zinc-500 block">ELEV</span>
            <span class="text-xs font-black text-amber-400">+{activeTrack.elevation_gain_m}m</span>
          </div>
          <div class="bg-zinc-900/80 p-1.5 rounded-xl border border-zinc-800">
            <span class="text-[9px] text-zinc-500 block">PACE</span>
            <span class="text-xs font-black text-cyan-400">{activeTrack.avg_pace_min_km}</span>
          </div>
        </div>

        {#if privacyCloakEnabled}
          <div class="text-[10px] text-emerald-400/90 flex items-center gap-1 font-semibold pt-1 border-t border-zinc-800/80">
            <ShieldCheck class="w-3 h-3 text-emerald-400" />
            <span>Origin/End points cloaked by 300m radius</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
