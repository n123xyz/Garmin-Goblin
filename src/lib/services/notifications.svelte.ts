import toast from 'svelte-french-toast';
import { 
  isPermissionGranted, 
  requestPermission, 
  sendNotification 
} from '@tauri-apps/plugin-notification';
import type { NotificationSettings, GarminBiometrics, GoblinQuest } from '$lib/types/garmin';
import { goblinState, setGoblinSpeech } from '$lib/state/goblin.svelte';

const STORAGE_KEY = 'garmin_goblin_notifications';

export const defaultNotificationSettings: NotificationSettings = {
  enabled: true,
  step_milestones: true,
  body_battery_alerts: true,
  hydration_reminders: true,
  sleep_bedtime_alerts: true,
  quest_alerts: true,
  stress_alerts: true,
  sound_enabled: true,
  frequency_minutes: 120,
};

export const notificationState = $state<{
  settings: NotificationSettings;
  permission: NotificationPermission | 'unsupported';
  isInitialized: boolean;
}>({
  settings: { ...defaultNotificationSettings },
  permission: 'default',
  isInitialized: false,
});

// Cooldown tracker to prevent duplicate notification spamming (timestamps in ms)
const lastAlertTimestamps: Record<string, number> = {};

/**
 * Play a light, crisp two-tone Goblin alert chime via Web Audio (Zero memory / zero external files)
 */
export function playGoblinChime(): void {
  if (!notificationState.settings.sound_enabled) return;
  try {
    const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
    if (!AudioCtx) return;
    const ctx = new AudioCtx();
    const now = ctx.currentTime;

    // Friendly 2-tone melodic chirp (587 Hz D5 -> 880 Hz A5)
    const freqs = [587.33, 880.0];
    freqs.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now + i * 0.12);

      gain.gain.setValueAtTime(0.18, now + i * 0.12);
      gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.12 + 0.35);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(now + i * 0.12);
      osc.stop(now + i * 0.12 + 0.35);
    });
  } catch (err) {
    console.warn('[Notifications] Audio chime unavailable:', err);
  }
}

/**
 * Load notification settings from localStorage and check OS permission
 */
export function loadNotificationSettings(): NotificationSettings {
  if (typeof window === 'undefined') return defaultNotificationSettings;

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      notificationState.settings = { ...defaultNotificationSettings, ...parsed };
    }
  } catch (e) {
    console.warn('Failed to load notification settings:', e);
  }

  isPermissionGranted()
    .then((granted) => {
      notificationState.permission = granted ? 'granted' : 'default';
    })
    .catch(() => {
      try {
        if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
          notificationState.permission = (window as any).Notification?.permission || 'default';
        } else {
          notificationState.permission = 'default';
        }
      } catch {
        notificationState.permission = 'default';
      }
    });

  return notificationState.settings;
}

/**
 * Save notification settings to localStorage
 */
export function saveNotificationSettings(newSettings: NotificationSettings): void {
  notificationState.settings = { ...newSettings };
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(notificationState.settings));
      toast.success('Notification preferences saved!');
    } catch (e) {
      console.warn('Failed to save notification settings:', e);
    }
  }
}

/**
 * Request native OS Notification permission from user
 */
export async function requestNotificationPermission(): Promise<boolean> {
  try {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }
    notificationState.permission = permissionGranted ? 'granted' : 'denied';
    if (permissionGranted) {
      toast.success('System notifications enabled for Gribble!');
      return true;
    } else {
      toast.error('Notification permission was denied or dismissed');
      return false;
    }
  } catch (e) {
    console.error('Error requesting notification permission via Tauri plugin:', e);
    try {
      if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
        const perm = await (window as any).Notification.requestPermission();
        notificationState.permission = perm;
        if (perm === 'granted') {
          toast.success('System notifications enabled for Gribble!');
          return true;
        }
      }
    } catch {}
    toast.error('Notification permission could not be granted');
    return false;
  }
}

/**
 * Send a notification from Gribble (OS notification + in-app toast + speech bubble update)
 * Zero AI involved: pure rule-based, deterministic, zero model memory footprint.
 */
export function sendGribbleNotification(options: {
  title: string;
  body: string;
  category: keyof NotificationSettings | 'test';
  icon?: string;
}): boolean {
  if (!notificationState.settings.enabled) return false;

  // Verify specific category is enabled (unless test)
  if (options.category !== 'test') {
    const isCategoryEnabled = notificationState.settings[options.category];
    if (isCategoryEnabled === false) return false;
  }

  // 1. Play light notification chime
  playGoblinChime();

  // 2. Update Gribble avatar speech bubble immediately in UI
  setGoblinSpeech(options.body, 8000);

  // 3. Dispatch in-app toast with Gribble icon
  toast(options.body, {
    icon: options.icon || '👹',
    duration: 6000,
    style: 'border: 1px solid rgba(16, 185, 129, 0.4); background: #09090b; color: #f4f4f5; font-size: 13px; font-weight: 600;',
  });

  // 4. Dispatch Native OS Notification via Tauri Notification Plugin
  try {
    sendNotification({
      title: options.title,
      body: options.body,
    });
  } catch (err) {
    console.warn('Tauri native notification dispatch error:', err);
    try {
      if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
        if ((window as any).Notification?.permission === 'granted') {
          new (window as any).Notification(options.title, {
            body: options.body,
            icon: '/icons/128x128.png',
            tag: `garmin-goblin-${options.category}`,
          });
        }
      }
    } catch (fallbackErr) {
      console.warn('Web notification dispatch fallback error:', fallbackErr);
    }
  }

  return true;
}

/**
 * Evaluates live biometrics and quests against deterministic trigger rules.
 * Uses cooldown timers to prevent repeating alerts within the user's frequency window.
 */
export function evaluateBiometricsForNotifications(
  bio: GarminBiometrics | null,
  quests: GoblinQuest[] = []
): void {
  if (!notificationState.settings.enabled || !bio) return;

  const now = Date.now();
  const cooldownMs = (notificationState.settings.frequency_minutes || 120) * 60 * 1000;
  const goblinName = goblinState.profile?.name || 'Gribble';

  // 1. Quest Completion Alert
  if (notificationState.settings.quest_alerts) {
    const unclaimed = quests.filter((q) => q.is_completed && !q.is_claimed);
    if (unclaimed.length > 0) {
      const last = lastAlertTimestamps['quest_ready'] || 0;
      if (now - last > 60 * 60 * 1000) { // 1h cooldown
        lastAlertTimestamps['quest_ready'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Quest Ready to Claim!`,
          body: `📜 GRAH! You completed "${unclaimed[0].title}"! Claim your ${unclaimed[0].reward_xp} XP & ${unclaimed[0].reward_gold} Gold!`,
          category: 'quest_alerts',
          icon: '📜',
        });
        return;
      }
    }
  }

  // 2. Step Goal Milestones
  if (notificationState.settings.step_milestones && bio.steps > 0) {
    const goal = bio.step_goal || 10000;
    const pct = Math.round((bio.steps / goal) * 100);

    if (pct >= 100) {
      const last = lastAlertTimestamps['step_100'] || 0;
      if (now - last > 12 * 60 * 60 * 1000) { // once per 12h
        lastAlertTimestamps['step_100'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Step Goal Smashed!`,
          body: `👟 BY THE CAVERN! You crushed 100% of your daily step goal (${bio.steps.toLocaleString()} steps)! Gribble is doing a victory dance!`,
          category: 'step_milestones',
          icon: '👟',
        });
        return;
      }
    } else if (pct >= 50) {
      const last = lastAlertTimestamps['step_50'] || 0;
      if (now - last > 12 * 60 * 60 * 1000) { // once per 12h
        lastAlertTimestamps['step_50'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Halfway There!`,
          body: `👣 You reached 50% of your step goal (${bio.steps.toLocaleString()} steps)! Keep marching, human!`,
          category: 'step_milestones',
          icon: '👣',
        });
        return;
      }
    }
  }

  // 3. Body Battery Alerts
  if (notificationState.settings.body_battery_alerts && bio.body_battery > 0) {
    if (bio.body_battery <= 25) {
      const last = lastAlertTimestamps['body_battery_low'] || 0;
      if (now - last > cooldownMs) {
        lastAlertTimestamps['body_battery_low'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Low Energy Reserves!`,
          body: `⚠️ Warning! Your Body Battery is at ${bio.body_battery}%! Rest up and avoid heavy dungeon strain before you crash!`,
          category: 'body_battery_alerts',
          icon: '⚡',
        });
        return;
      }
    } else if (bio.body_battery >= 85) {
      const last = lastAlertTimestamps['body_battery_high'] || 0;
      if (now - last > cooldownMs) {
        lastAlertTimestamps['body_battery_high'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Peak Stamina Surge!`,
          body: `⚡ Energy is charged to ${bio.body_battery}%! Perfect window to conquer a workout quest today!`,
          category: 'body_battery_alerts',
          icon: '⚡',
        });
        return;
      }
    }
  }

  // 4. Stress Spike Alert
  if (notificationState.settings.stress_alerts && bio.stress_level >= 75) {
    const last = lastAlertTimestamps['stress_spike'] || 0;
    if (now - last > cooldownMs) {
      lastAlertTimestamps['stress_spike'] = now;
      sendGribbleNotification({
        title: `${goblinName}: High Autonomic Stress!`,
        body: `🧘 High stress level detected (${bio.stress_level}/100). Take 2 minutes for Box Breathing or a Zen pause!`,
        category: 'stress_alerts',
        icon: '🧘',
      });
      return;
    }
  }

  // 5. Circadian Bedtime Wind-Down Alert
  if (notificationState.settings.sleep_bedtime_alerts) {
    const currentHour = new Date().getHours();
    const currentMinute = new Date().getMinutes();
    // Evening wind-down window (e.g. 10:00 PM - 10:45 PM)
    if (currentHour === 22 && currentMinute <= 45) {
      const last = lastAlertTimestamps['sleep_winddown'] || 0;
      if (now - last > 18 * 60 * 60 * 1000) { // once per night
        lastAlertTimestamps['sleep_winddown'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Bedtime Wind-Down Window!`,
          body: `🌙 Target bedtime approaches! Put down glowing screens, drink herbal tea, and prepare for restorative sleep!`,
          category: 'sleep_bedtime_alerts',
          icon: '🌙',
        });
        return;
      }
    }
  }

  // 6. Hydration & Nutrition Check-in
  if (notificationState.settings.hydration_reminders) {
    const currentHour = new Date().getHours();
    // Midday hydration prompt (1:00 PM - 3:00 PM)
    if (currentHour >= 13 && currentHour <= 15) {
      const last = lastAlertTimestamps['hydration_midday'] || 0;
      if (now - last > 8 * 60 * 60 * 1000) {
        lastAlertTimestamps['hydration_midday'] = now;
        sendGribbleNotification({
          title: `${goblinName}: Hydration Check!`,
          body: `💧 Drink 250ml of clean water right now to keep your goblin stats at peak performance!`,
          category: 'hydration_reminders',
          icon: '💧',
        });
        return;
      }
    }
  }
}

/**
 * Send an immediate test notification to verify OS notification, toast, audio chime, and speech bubble
 */
export function sendTestNotification(): void {
  const goblinName = goblinState.profile?.name || 'Gribble';
  sendGribbleNotification({
    title: `${goblinName}: Testing Goblin Telemetry!`,
    body: `👹 GRAH! The goblin communication link is operational! I can now send you alerts for steps, energy, and quests!`,
    category: 'test',
    icon: '👹',
  });
}

let tickerInterval: any = null;

/**
 * Start the background notification rule evaluator ticker (runs every 60s, completely zero AI / low CPU)
 */
export function startNotificationTicker(
  getBiometrics: () => GarminBiometrics | null,
  getQuests: () => GoblinQuest[]
): void {
  if (typeof window === 'undefined') return;
  loadNotificationSettings();

  if (tickerInterval) clearInterval(tickerInterval);

  tickerInterval = setInterval(() => {
    try {
      const bio = getBiometrics();
      const quests = getQuests();
      evaluateBiometricsForNotifications(bio, quests);
    } catch (e) {
      console.warn('[Notifications] Error in ticker evaluation:', e);
    }
  }, 60 * 1000); // Check every 60 seconds
}
