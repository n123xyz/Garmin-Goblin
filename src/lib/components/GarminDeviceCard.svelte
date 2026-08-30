<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    Watch,
    Bluetooth,
    RefreshCw,
    Battery,
    CheckCircle2,
    Radio,
    Sparkles,
    ShieldCheck,
    CloudSun,
    AlertCircle,
    X,
    Flame,
    Zap,
    FileUp,
    FolderOpen
  } from 'lucide-svelte';
  import toast from 'svelte-french-toast';
  import type { GarminDeviceInfo, GarminOffloadSummary, GarminUsbDeviceInfo, GarminMtpSyncSummary } from '$lib/types/garmin';
  import { loadActivities, loadLatestBiometrics, loadBiometricsHistory } from '$lib/state/biometrics.svelte';
  import { loadGoblinProfile } from '$lib/state/goblin.svelte';

  interface Props {
    onSyncComplete?: (summary: GarminOffloadSummary) => void;
  }

  let { onSyncComplete }: Props = $props();

  let pairedDevice = $state<GarminDeviceInfo | null>(null);
  let usbDevice = $state<GarminUsbDeviceInfo | null>(null);
  let isScanning = $state(false);
  let isSyncing = $state(false);
  let isSyncingUsb = $state(false);
  let isCheckingUsb = $state(false);
  let showPairModal = $state(false);
  let discoveredDevices = $state<GarminDeviceInfo[]>([]);
  let lastOffloadSummary = $state<GarminOffloadSummary | null>(null);

  onMount(async () => {
    await Promise.all([
      fetchDeviceStatus(),
      checkUsbStatus()
    ]);
  });

  export async function fetchDeviceStatus() {
    try {
      const dev = await invoke<GarminDeviceInfo | null>('get_garmin_device_status');
      pairedDevice = dev;
    } catch (e) {
      console.warn('Failed to load Garmin device status:', e);
    }
  }

  async function handleScanDevices() {
    isScanning = true;
    showPairModal = true;
    discoveredDevices = [];
    try {
      let list: GarminDeviceInfo[] = [];
      let statusInfo = '';

      // 1. Native Android Kotlin BLE Scanner
      try {
        const res = await invoke<{ devices?: any[]; status?: string }>('plugin:litert|scan_ble_devices');
        if (res && res.status) {
          statusInfo = res.status;
        }
        if (res && res.devices && res.devices.length > 0) {
          list = res.devices.map((d: any) => ({
            device_id: d.deviceId || `ble-${d.macAddress.replace(/:/g, '').toLowerCase()}`,
            device_name: d.deviceName || 'Garmin Watch',
            mac_address: d.macAddress,
            rssi: d.rssi ?? -50,
            is_paired: pairedDevice?.mac_address?.toLowerCase() === d.macAddress?.toLowerCase(),
            is_connected: pairedDevice?.mac_address?.toLowerCase() === d.macAddress?.toLowerCase(),
            battery_level: undefined,
            last_sync_time: undefined,
            pending_fit_files: 0
          }));
        }
      } catch (nativeErr) {
        console.warn('Native BLE scan error:', nativeErr);
      }

      // Fallback to desktop / backend scan if 0 devices found
      if (list.length === 0) {
        try {
          const backendList = await invoke<GarminDeviceInfo[]>('scan_garmin_devices');
          if (backendList && backendList.length > 0) {
            list = backendList;
          }
        } catch (backendErr) {
          console.warn('Backend scan error:', backendErr);
        }
      }

      discoveredDevices = list;

      if (list.length > 0) {
        toast.success(`Discovered ${list.length} Bluetooth device(s)!`);
      } else {
        toast(statusInfo || 'No Bluetooth devices detected. Ensure watch Bluetooth is enabled.', { icon: '🔍' });
      }
    } catch (e) {
      toast.error('Bluetooth scan failed: ' + e);
    } finally {
      isScanning = false;
    }
  }

  async function handlePairDevice(device: GarminDeviceInfo) {
    try {
      const paired = await invoke<GarminDeviceInfo>('pair_garmin_device', {
        deviceId: device.device_id,
        deviceName: device.device_name,
        macAddress: device.mac_address
      });
      pairedDevice = paired;
      showPairModal = false;
      toast.success(`Paired with ${paired.device_name}!`);
    } catch (e) {
      toast.error('Pairing failed: ' + e);
    }
  }

  async function handleUnpair() {
    try {
      await invoke('unpair_garmin_device');
      pairedDevice = null;
      toast.success('Garmin watch unpaired.');
    } catch (e) {
      toast.error('Unpair failed: ' + e);
    }
  }

  export async function checkUsbStatus() {
    isCheckingUsb = true;
    try {
      const usb = await invoke<GarminUsbDeviceInfo>('scan_usb_mtp_status');
      usbDevice = usb;
    } catch (e) {
      console.warn('Failed to check USB MTP status:', e);
    } finally {
      isCheckingUsb = false;
    }
  }

  export async function handleSyncUsbMtp() {
    if (isSyncingUsb) return;
    isSyncingUsb = true;
    try {
      // 1. Scan for USB device
      const status = await invoke<GarminUsbDeviceInfo>('scan_usb_mtp_status');
      usbDevice = status;
      if (!status.is_attached) {
        toast('No Garmin watch detected on USB. Connect your watch via USB cable and ensure it is unlocked.', { icon: '🔌' });
        return;
      }

      // 2. Check permission
      if (!status.has_permission) {
        toast('Requesting Android USB permission...', { icon: '🔑' });
        const granted = await invoke<boolean>('request_usb_mtp_permission');
        if (!granted) {
          toast.error('USB permission denied. Cannot offload via MTP.');
          return;
        }
      }

      toast.loading('Offloading Garmin FIT workouts over USB MTP...', { id: 'mtp-sync' });

      // 3. Perform MTP offload and ingestion
      const summary = await invoke<GarminMtpSyncSummary>('sync_garmin_mtp_device', { forcePull: false });
      toast.dismiss('mtp-sync');

      if (summary.synced_count > 0) {
        toast.success(`⚡ MTP Sync Complete! Ingested ${summary.synced_count} file(s) (+${summary.xp_earned} XP)!`);
      } else {
        toast.success(`⚡ Garmin watch is up to date! (${(summary.total_bytes / 1024).toFixed(1)} KB checked)`);
      }

      lastOffloadSummary = {
        device_name: summary.device_name,
        synced_activities: summary.activities,
        xp_earned: summary.xp_earned,
        gold_earned: summary.gold_earned,
        biometrics_updated: true,
        weather_streamed: false,
        weather_condition: 'USB MTP',
        last_sync_timestamp: summary.last_sync_timestamp,
        status_message: summary.status_message
      };

      // Reload fresh database state
      await loadLatestBiometrics();
      await loadBiometricsHistory(14);
      await loadActivities(10);
      await loadGoblinProfile();
      await fetchDeviceStatus();
      await checkUsbStatus();

      if (onSyncComplete && lastOffloadSummary) {
        onSyncComplete(lastOffloadSummary);
      }
    } catch (e) {
      toast.dismiss('mtp-sync');
      toast.error('USB MTP Sync failed: ' + e);
    } finally {
      isSyncingUsb = false;
    }
  }

  export async function handleSyncAndOffload() {
    if (isSyncing) return;
    isSyncing = true;
    try {
      let nativeStatus = '';
      let nativeBattery: number | undefined = undefined;
      let fitCount = 0;

      if (pairedDevice?.mac_address) {
        const res = await invoke<{ success: boolean; statusMessage: string; batteryLevel?: number; fitFiles?: string[]; syncedActivitiesCount?: number }>('plugin:litert|sync_ble_device', {
          payload: {
            macAddress: pairedDevice.mac_address,
            forcePullAll: false
          },
          macAddress: pairedDevice.mac_address
        });
        if (!res || !res.success) {
          throw new Error(res?.statusMessage || 'BLE sync failed to connect');
        }
        if (typeof res.batteryLevel === 'number') {
          nativeBattery = res.batteryLevel;
        }
        if (res.statusMessage) {
          nativeStatus = res.statusMessage;
        }
        if (res.fitFiles && res.fitFiles.length > 0) {
          for (const fitB64 of res.fitFiles) {
            try {
              const imported = await invoke<any>('import_fit_base64', {
                base64Data: fitB64,
                base64_data: fitB64
              });
              if (imported && imported.success) {
                if (imported.activity?.duration_sec > 0 || imported.activity?.distance_meters > 0) {
                  fitCount++;
                  toast.success(`🏅 Synced ${imported.activity?.title || 'Garmin Workout'} (${Math.round(imported.activity.duration_sec / 60)} min)! +${imported.xp_earned} XP`);
                } else {
                  toast.success(`📊 Updated ${imported.activity?.title || 'Garmin Biometrics'}`);
                }
              }
            } catch (fitErr) {
              console.error('Error ingesting synced FIT file:', fitErr);
              toast.error(`FIT ingest: ${fitErr}`);
            }
          }
        }
      }

      const summary = await invoke<GarminOffloadSummary>('sync_and_offload_garmin', {
        batteryLevel: nativeBattery
      });
      lastOffloadSummary = summary;

      if (fitCount > 0) {
        toast.success(`⚡ Synced ${fitCount} recorded workout(s)!`);
      } else {
        toast.success(`⚡ Garmin sync complete! Biometrics updated.`);
      }

      // Reload fresh database state
      await loadLatestBiometrics();
      await loadBiometricsHistory(14);
      await loadActivities(10);
      await loadGoblinProfile();
      await fetchDeviceStatus();

      if (onSyncComplete) {
        onSyncComplete(summary);
      }
    } catch (e) {
      toast.error('Sync & Offload failed: ' + e);
    } finally {
      isSyncing = false;
    }
  }
</script>

<div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-amber-950/20 border border-amber-500/30 rounded-3xl p-5 md:p-6 shadow-xl space-y-4">
  <!-- Header with Device Info & Primary Action Buttons -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800/80 pb-4">
    <div class="flex items-center gap-3.5">
      <div class="w-12 h-12 rounded-2xl bg-amber-500/20 text-amber-400 border border-amber-500/40 flex items-center justify-center shadow-[0_0_15px_rgba(245,158,11,0.2)]">
        <Watch class="w-6 h-6" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="text-lg font-bold text-zinc-100">
            {pairedDevice ? pairedDevice.device_name : 'No Garmin Watch Paired'}
          </h3>
          <span class="flex items-center gap-1 text-[10px] px-2.5 py-0.5 rounded-full font-extrabold uppercase tracking-wider {pairedDevice?.is_connected ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-zinc-800 text-zinc-400 border border-zinc-700'}">
            <span class="w-1.5 h-1.5 rounded-full {pairedDevice?.is_connected ? 'bg-emerald-400 animate-pulse' : 'bg-zinc-500'}"></span>
            {pairedDevice?.is_connected ? 'BLE Connected' : 'BLE Standby'}
          </span>
          <span class="flex items-center gap-1 text-[10px] px-2.5 py-0.5 rounded-full font-extrabold uppercase tracking-wider {usbDevice?.is_attached ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30' : 'bg-zinc-800/80 text-zinc-500 border border-zinc-700/50'}">
            <span class="w-1.5 h-1.5 rounded-full {usbDevice?.is_attached ? 'bg-cyan-400 animate-pulse' : 'bg-zinc-600'}"></span>
            {usbDevice?.is_attached ? 'USB (MTP) Connected' : 'USB Cable Standby'}
          </span>
        </div>
        <div class="flex items-center gap-3 text-xs text-zinc-400 mt-1 font-mono">
          {#if pairedDevice}
            <span>MAC: {pairedDevice.mac_address}</span>
            {#if pairedDevice.battery_level != null}
              <span>•</span>
              <span class="flex items-center gap-1 text-emerald-400">
                <Battery class="w-3.5 h-3.5" />
                {pairedDevice.battery_level}%
              </span>
            {/if}
            <span>•</span>
            <span>Sync: {pairedDevice.last_sync_time || 'Never'}</span>
          {:else}
            <span>Pair your Garmin watch over Bluetooth or plug in via USB cable to sync</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Pair & Offload Action Buttons -->
    <div class="flex items-center gap-2 flex-wrap">
      <button
        onclick={handleScanDevices}
        disabled={isScanning || isSyncing || isSyncingUsb}
        class="px-3.5 py-2 rounded-xl bg-zinc-950 hover:bg-zinc-800 text-zinc-200 border border-zinc-700 font-bold text-xs transition-all cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
      >
        <Bluetooth class="w-3.5 h-3.5 text-blue-400 {isScanning ? 'animate-spin' : ''}" />
        <span>{pairedDevice ? 'Change Watch' : 'Pair BLE'}</span>
      </button>

      {#if pairedDevice}
        <button
          onclick={handleSyncAndOffload}
          disabled={isSyncing || isSyncingUsb}
          class="px-4 py-2 rounded-xl bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-zinc-950 font-black text-xs shadow-[0_0_15px_rgba(245,158,11,0.3)] transition-all cursor-pointer disabled:opacity-50 flex items-center gap-2"
        >
          <RefreshCw class="w-4 h-4 {isSyncing ? 'animate-spin' : ''}" />
          <span>{isSyncing ? 'Syncing BLE...' : 'Sync BLE'}</span>
        </button>
      {/if}

      <!-- USB MTP Sync Option -->
      <button
        onclick={handleSyncUsbMtp}
        disabled={isSyncing || isSyncingUsb}
        class="px-4 py-2 rounded-xl {usbDevice?.is_attached ? 'bg-gradient-to-r from-cyan-500 to-emerald-500 hover:from-cyan-400 hover:to-emerald-400 text-zinc-950 font-black shadow-[0_0_15px_rgba(6,182,212,0.35)]' : 'bg-zinc-950 hover:bg-zinc-800 text-zinc-200 border border-cyan-500/40 font-bold'} text-xs transition-all cursor-pointer disabled:opacity-50 flex items-center gap-2"
      >
        <FolderOpen class="w-4 h-4 {isSyncingUsb ? 'animate-spin' : (usbDevice?.is_attached ? 'text-zinc-950' : 'text-cyan-400')}" />
        <span>{isSyncingUsb ? 'Syncing MTP...' : (usbDevice?.is_attached ? 'Sync USB (MTP)' : 'Sync USB Cable')}</span>
      </button>
    </div>
  </div>

  {#if usbDevice?.is_attached}
    <div class="bg-gradient-to-r from-cyan-950/40 via-zinc-950 to-zinc-900 border border-cyan-500/30 rounded-2xl p-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-2.5 text-xs">
      <div class="flex items-center gap-2.5 text-cyan-200">
        <span class="relative flex h-2.5 w-2.5">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75"></span>
          <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-cyan-500"></span>
        </span>
        <div>
          <span class="font-bold text-zinc-100">{usbDevice.device_name} Attached (USB MTP)</span>
          <span class="text-[11px] text-zinc-400 block sm:inline sm:ml-2">Fast hardware offload (~20 MB/s) available</span>
        </div>
      </div>
      <button
        onclick={handleSyncUsbMtp}
        disabled={isSyncingUsb}
        class="px-3.5 py-1.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-zinc-950 font-black text-xs transition-all cursor-pointer shrink-0 self-start sm:self-auto flex items-center gap-1.5 shadow-[0_0_10px_rgba(6,182,212,0.3)]"
      >
        <Zap class="w-3.5 h-3.5" />
        <span>{isSyncingUsb ? 'Offloading...' : 'Instant MTP Sync'}</span>
      </button>
    </div>
  {/if}

  <!-- Offload / Auto-Weather Telemetry Status Ribbon -->
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3.5 flex items-center gap-3">
      <div class="w-8 h-8 rounded-xl bg-amber-500/10 text-amber-400 flex items-center justify-center">
        <Flame class="w-4 h-4" />
      </div>
      <div>
        <span class="text-[10px] uppercase font-bold text-zinc-500 block">Activity Offload</span>
        <span class="text-xs font-bold text-zinc-200">Auto-ingest FIT workouts to Goblin XP</span>
      </div>
    </div>

    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3.5 flex items-center gap-3">
      <div class="w-8 h-8 rounded-xl bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
        <Zap class="w-4 h-4" />
      </div>
      <div>
        <span class="text-[10px] uppercase font-bold text-zinc-500 block">Biometrics Sync</span>
        <span class="text-xs font-bold text-zinc-200">HR, Steps, Sleep, Stress & Body Battery</span>
      </div>
    </div>

    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-3.5 flex items-center gap-3">
      <div class="w-8 h-8 rounded-xl bg-cyan-500/10 text-cyan-400 flex items-center justify-center">
        <CloudSun class="w-4 h-4" />
      </div>
      <div>
        <span class="text-[10px] uppercase font-bold text-zinc-500 block">Weather Broadcast</span>
        <span class="text-xs font-bold text-zinc-200">Auto-streamed to watch during offload</span>
      </div>
    </div>
  </div>

  {#if lastOffloadSummary}
    <div class="bg-amber-500/10 border border-amber-500/30 rounded-2xl p-3.5 flex items-center justify-between text-xs">
      <div class="flex items-center gap-2 text-amber-200">
        <Sparkles class="w-4 h-4 text-amber-400 shrink-0" />
        <span>{lastOffloadSummary.status_message}</span>
      </div>
      <span class="text-[10px] text-zinc-500 font-mono shrink-0 ml-2">{lastOffloadSummary.last_sync_timestamp}</span>
    </div>
  {/if}
</div>

<!-- BLE Device Pairing Modal -->
{#if showPairModal}
  <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-4 animate-in fade-in zoom-in duration-200">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <Bluetooth class="w-5 h-5 text-blue-400" />
          <h3 class="text-base font-bold text-zinc-100">Pair Garmin Watch (BLE)</h3>
        </div>
        <button
          onclick={() => showPairModal = false}
          class="p-1 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <p class="text-xs text-zinc-400">
        Searching for Garmin watches broadcasting on the Garmin GFDI service UUID. Ensure Bluetooth is enabled on your watch.
      </p>

      <div class="space-y-2.5 max-h-64 overflow-y-auto pr-1">
        {#if discoveredDevices.length === 0 && isScanning}
          <div class="p-6 text-center text-zinc-500 text-xs flex flex-col items-center gap-2">
            <Radio class="w-6 h-6 animate-pulse text-blue-400" />
            <span>Scanning for nearby Garmin Bluetooth Low Energy devices...</span>
          </div>
        {:else if discoveredDevices.length === 0}
          <div class="p-6 text-center text-zinc-500 text-xs">
            No Garmin watches found. Press Rescan to search again.
          </div>
        {:else}
          {#each discoveredDevices as dev}
            <div class="bg-zinc-950 border border-zinc-800 hover:border-amber-500/50 rounded-2xl p-3.5 flex items-center justify-between transition-colors">
              <div>
                <div class="flex items-center gap-2">
                  <span class="text-sm font-bold text-zinc-200">{dev.device_name}</span>
                  {#if dev.is_paired}
                    <span class="text-[9px] px-2 py-0.5 rounded-full font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30">
                      PAIRED
                    </span>
                  {/if}
                </div>
                <div class="flex items-center gap-2 text-[11px] text-zinc-500 font-mono mt-0.5">
                  <span>{dev.mac_address}</span>
                  <span>•</span>
                  <span>RSSI: {dev.rssi} dBm</span>
                  <span>•</span>
                  <span>Battery: {dev.battery_level}%</span>
                </div>
              </div>

              <button
                onclick={() => handlePairDevice(dev)}
                class="px-3 py-1.5 rounded-xl bg-amber-500 hover:bg-amber-400 text-zinc-950 font-black text-xs transition-all cursor-pointer"
              >
                {dev.is_paired ? 'Reconnect' : 'Pair'}
              </button>
            </div>
          {/each}
        {/if}
      </div>

      <div class="flex justify-between items-center pt-3 border-t border-zinc-800">
        {#if pairedDevice}
          <button
            onclick={handleUnpair}
            class="text-xs text-rose-400 hover:underline cursor-pointer"
          >
            Unpair Current Watch
          </button>
        {:else}
          <div></div>
        {/if}

        <div class="flex gap-2">
          <button
            onclick={handleScanDevices}
            disabled={isScanning}
            class="px-3.5 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-bold transition-all cursor-pointer disabled:opacity-50 flex items-center gap-1"
          >
            <RefreshCw class="w-3.5 h-3.5 {isScanning ? 'animate-spin' : ''}" />
            <span>Rescan</span>
          </button>
          <button
            onclick={() => showPairModal = false}
            class="px-4 py-1.5 rounded-xl bg-zinc-950 text-zinc-300 text-xs font-bold hover:bg-zinc-800 transition-all cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
