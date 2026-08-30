import { invoke } from '@tauri-apps/api/core';
import type { GoblinProfile, GoblinQuest } from '$lib/types/garmin';
import toast from 'svelte-french-toast';

export const goblinState = $state<{
  profile: GoblinProfile | null;
  quests: GoblinQuest[];
  isLoading: boolean;
  activeSpeechBubble: string | null;
}>({
  profile: null,
  quests: [],
  isLoading: false,
  activeSpeechBubble: null,
});

let speechTimeout: any = null;

export function setGoblinSpeech(message: string, durationMs = 6000) {
  goblinState.activeSpeechBubble = message;
  if (speechTimeout) clearTimeout(speechTimeout);
  speechTimeout = setTimeout(() => {
    goblinState.activeSpeechBubble = null;
  }, durationMs);
}

export async function loadGoblinProfile() {
  goblinState.isLoading = true;
  try {
    const profile = await invoke<GoblinProfile>('get_goblin_profile');
    goblinState.profile = profile;
  } catch (e) {
    console.error('Failed to load goblin profile:', e);
  } finally {
    goblinState.isLoading = false;
  }
}

export async function loadQuests() {
  try {
    const quests = await invoke<GoblinQuest[]>('get_quests');
    goblinState.quests = quests || [];
  } catch (e) {
    console.error('Failed to load quests:', e);
    goblinState.quests = [];
  }
}

export async function claimQuest(questId: number) {
  try {
    const updatedProfile = await invoke<GoblinProfile>('claim_quest_reward', { questId });
    goblinState.profile = updatedProfile;
    toast.success(`Quest claimed! +XP & +Gold awarded!`);
    await loadQuests();
    setGoblinSpeech(`HEHEHE! Gold claimed! My power grows, mortal!`);
  } catch (e) {
    toast.error('Failed to claim quest: ' + e);
  }
}
