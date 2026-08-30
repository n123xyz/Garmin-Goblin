import { invoke } from '@tauri-apps/api/core';
import type { CbtThoughtRecord, CbtThoughtRecordInput, CbtReframeResult } from '$lib/types/therapy';
import type { ChatMessage, ChatResponse } from '$lib/types/garmin';
import toast from 'svelte-french-toast';

export const therapyState = $state({
  // In-memory strictly ephemeral chat history (NEVER stored to DB or disk)
  messages: [] as ChatMessage[],
  isSending: false,
  activeTab: 'chat' as 'chat' | 'cbt' | 'somatic' | 'vault',

  // CBT Records
  cbtRecords: [] as CbtThoughtRecord[],
  isLoadingRecords: false,
  isReframing: false,
});

export async function sendTherapyChatMessage(content: string): Promise<string | null> {
  const trimmed = content.trim();
  if (!trimmed || therapyState.isSending) return null;

  const userMsg: ChatMessage = { role: 'user', content: trimmed };
  therapyState.messages = [...therapyState.messages, userMsg];
  therapyState.isSending = true;

  try {
    const resp = await invoke<ChatResponse>('chat_with_therapy_goblin', {
      history: therapyState.messages,
    });

    const assistantMsg: ChatMessage = {
      role: 'assistant',
      content: resp.response,
      goblinMood: resp.goblin_mood,
    };

    therapyState.messages = [...therapyState.messages, assistantMsg];
    return resp.response;
  } catch (err) {
    console.error('Failed to chat in therapy mode:', err);
    toast.error('Therapy response error: ' + err);
    return null;
  } finally {
    therapyState.isSending = false;
  }
}

export function clearTherapyChat() {
  therapyState.messages = [];
  toast.success('Therapy chat memory wiped clean.');
}

export async function loadCbtRecords(date?: string) {
  therapyState.isLoadingRecords = true;
  try {
    const list = await invoke<CbtThoughtRecord[]>('get_cbt_thought_records', {
      date: date || null,
      limit: 50,
    });
    therapyState.cbtRecords = list || [];
  } catch (err) {
    console.error('Failed to load CBT records:', err);
  } finally {
    therapyState.isLoadingRecords = false;
  }
}

export async function saveCbtRecord(record: CbtThoughtRecordInput): Promise<CbtThoughtRecord | null> {
  try {
    const saved = await invoke<CbtThoughtRecord>('save_cbt_thought_record', { record });
    therapyState.cbtRecords = [saved, ...therapyState.cbtRecords];
    toast.success('CBT thought reframing saved to vault!');
    return saved;
  } catch (err) {
    console.error('Failed to save CBT record:', err);
    toast.error('Failed to save record: ' + err);
    return null;
  }
}

export async function deleteCbtRecord(id: number) {
  try {
    await invoke<boolean>('delete_cbt_thought_record', { id });
    therapyState.cbtRecords = therapyState.cbtRecords.filter((r) => r.id !== id);
    toast.success('Record deleted.');
  } catch (err) {
    console.error('Failed to delete CBT record:', err);
    toast.error('Failed to delete: ' + err);
  }
}

export async function requestCbtReframe(
  situation: string,
  automaticThought: string,
  distortions: string[]
): Promise<CbtReframeResult | null> {
  therapyState.isReframing = true;
  try {
    const result = await invoke<CbtReframeResult>('reframe_cbt_thought', {
      situation,
      automaticThought,
      distortions,
    });
    return result;
  } catch (err) {
    console.error('Failed to generate CBT reframe:', err);
    toast.error('Reframing error: ' + err);
    return null;
  } finally {
    therapyState.isReframing = false;
  }
}
