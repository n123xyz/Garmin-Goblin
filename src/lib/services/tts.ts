import { invoke } from '@tauri-apps/api/core';
import { settingsState } from '$lib/state/settings.svelte';

export let ttsAudioContext: AudioContext | null = null;
export let ttsPlaybackRate: number = 1.0;
let currentSourceNode: AudioBufferSourceNode | null = null;
let activeSessionId = 0;

/**
 * Initialize or resume the shared Web Audio Context
 */
export function getAudioContext(): AudioContext {
  if (!ttsAudioContext || ttsAudioContext.state === 'closed') {
    const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
    ttsAudioContext = new AudioCtx();
  }
  if (ttsAudioContext.state === 'suspended') {
    ttsAudioContext.resume();
  }
  return ttsAudioContext;
}

/**
 * Strips markdown, emojis, HTML/control tokens, and excessive whitespace
 */
export function sanitizeTTSInput(text: string): string {
  if (!text) return '';
  // Strip markdown formatting (*, _, ~, `, #)
  let sanitized = text.replace(/[*_~`#]/g, '');
  // Strip emojis
  sanitized = sanitized.replace(
    /[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F700}-\u{1F77F}\u{1F780}-\u{1F7FF}\u{1F800}-\u{1F8FF}\u{1F900}-\u{1F9FF}\u{1FA00}-\u{1FA6F}\u{1FA70}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/gu,
    ''
  );
  // Replace newlines with spaces
  sanitized = sanitized.replace(/\n+/g, ' ');
  // Strip LLM control tokens
  sanitized = sanitized.replace(/<start_of_turn>\s*model\s*/g, '');
  sanitized = sanitized.replace(/<start_of_turn>/g, '');
  sanitized = sanitized.replace(/<end_of_turn>/g, '');
  return sanitized.trim();
}

/**
 * Breaks long text into pairs of natural sentences using Intl.Segmenter
 */
export function chunkSentences(text: string, locale: string = 'en'): string[] {
  if (!text.trim()) return [];

  const isValidBcp47 = /^[a-z]{2,3}(-[a-zA-Z0-9]+)*$/i.test(locale);
  const segmenterLocale = isValidBcp47 ? locale : 'en';

  try {
    const sentenceSegmenter = new Intl.Segmenter(segmenterLocale, { granularity: 'sentence' });
    const sentences = Array.from(sentenceSegmenter.segment(text))
      .map((s) => s.segment.trim())
      .filter((s) => s.length > 0);

    return pairSegments(sentences);
  } catch (e) {
    const sentenceSegmenter = new Intl.Segmenter('en', { granularity: 'sentence' });
    const sentences = Array.from(sentenceSegmenter.segment(text))
      .map((s) => s.segment.trim())
      .filter((s) => s.length > 0);
    return pairSegments(sentences);
  }
}

function pairSegments(segments: string[]): string[] {
  const paired: string[] = [];
  for (let i = 0; i < segments.length; i += 2) {
    const chunk = i + 1 < segments.length ? segments[i] + ' ' + segments[i + 1] : segments[i];
    paired.push(chunk.trim());
  }
  return paired.filter((s) => s.length > 0);
}

/**
 * Play a gentle Zen singing bowl / yoga transition chime
 */
export function playZenChime(): void {
  try {
    const ctx = getAudioContext();
    const now = ctx.currentTime;

    // Harmonic singing bowl frequencies (432 Hz fundamental)
    const freqs = [432, 864, 1296];
    const gains = [0.25, 0.12, 0.04];

    freqs.forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const gainNode = ctx.createGain();

      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now);

      gainNode.gain.setValueAtTime(gains[idx], now);
      gainNode.gain.exponentialRampToValueAtTime(0.0001, now + 3.0);

      osc.connect(gainNode);
      gainNode.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 3.0);
    });
  } catch (err) {
    console.warn('[TTS] Could not play chime:', err);
  }
}

/**
 * Stop any active TTS audio playback immediately
 */
export function stopSpeech(): void {
  activeSessionId++; // Invalidate any running sequence loop

  try {
    if (currentSourceNode) {
      currentSourceNode.stop();
      currentSourceNode.disconnect();
      currentSourceNode = null;
    }
  } catch (e) {
    // Ignore if already disconnected
  }

  if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
    window.speechSynthesis.cancel();
  }
}

/**
 * Speaks text using SuperTonic Neural TTS on-device if available,
 * chunking sentences and decoding 16-bit PCM AudioBuffers,
 * with automatic fallback to Web Speech API.
 */
export async function speakGuidance(
  text: string,
  options?: { speed?: number; voiceStyle?: string; lang?: string }
): Promise<void> {
  stopSpeech();
  const sessionId = activeSessionId;

  const speed = options?.speed ?? ttsPlaybackRate ?? 1.0;
  const voiceStyle = options?.voiceStyle ?? settingsState.tts_voice_style ?? 'voice_styles/F1.json';
  const lang = options?.lang ?? 'en';

  const sanitized = sanitizeTTSInput(text);
  if (!sanitized) return;

  const chunks = chunkSentences(sanitized, lang);
  if (chunks.length === 0) return;

  // 1. Try SuperTonic Neural TTS for each chunk
  let supertonicSuccess = true;
  for (const chunk of chunks) {
    if (sessionId !== activeSessionId) return; // User stopped/skipped

    try {
      let res: any;
      try {
        res = await invoke('generate_supertonic_tts', {
          text: chunk,
          lang,
          speed,
          steps: 4,
          voice_style: voiceStyle,
        });
      } catch (rootErr) {
        // Fallback to plugin syntax if direct command fails
        res = await invoke('plugin:supertonic|generate_supertonic_tts', {
          payload: {
            text: chunk,
            lang,
            speed,
            steps: 4,
            voiceStyle,
          },
        });
      }

      const audioBytesArray = res?.audioBytes || res?.audio_bytes;
      const sampleRate = res?.sampleRate || res?.sample_rate || 24000;

      if (audioBytesArray && audioBytesArray.length > 0) {
        const ctx = getAudioContext();
        const rawBytes = new Uint8Array(audioBytesArray);
        const numSamples = Math.floor(rawBytes.byteLength / 2);
        
        const audioBuffer = ctx.createBuffer(1, numSamples, sampleRate);
        const channelData = audioBuffer.getChannelData(0);
        const dataView = new DataView(rawBytes.buffer, rawBytes.byteOffset, rawBytes.byteLength);

        for (let i = 0; i < numSamples; i++) {
          const sample16 = dataView.getInt16(i * 2, true); // little-endian
          channelData[i] = sample16 / 32768.0;
        }

        if (sessionId !== activeSessionId) return;

        const source = ctx.createBufferSource();
        source.buffer = audioBuffer;
        source.playbackRate.value = speed;
        source.connect(ctx.destination);
        currentSourceNode = source;

        await new Promise<void>((resolve) => {
          source.onended = () => {
            if (currentSourceNode === source) {
              currentSourceNode = null;
            }
            resolve();
          };
          source.start();
        });
      } else {
        supertonicSuccess = false;
        break;
      }
    } catch (err) {
      console.warn('[SuperTonic TTS] Inference failed or model downloading, falling back to Web Speech:', err);
      supertonicSuccess = false;
      break;
    }
  }

  if (supertonicSuccess) return;

  // 2. Web Speech API Fallback
  if (sessionId !== activeSessionId) return;
  return new Promise<void>((resolve) => {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
      resolve();
      return;
    }

    const utterance = new SpeechSynthesisUtterance(sanitized);
    utterance.rate = Math.max(0.8, Math.min(1.2, speed * 0.95));
    utterance.pitch = 1.0;

    const voices = window.speechSynthesis.getVoices();
    if (voices && voices.length > 0) {
      const preferred =
        voices.find(
          (v) =>
            v.lang.startsWith('en') &&
            (v.name.includes('Natural') || v.name.includes('Google') || v.name.includes('Samantha') || v.name.includes('Female'))
        ) || voices.find((v) => v.lang.startsWith('en'));
      if (preferred) {
        utterance.voice = preferred;
      }
    }

    utterance.onend = () => resolve();
    utterance.onerror = () => resolve();

    window.speechSynthesis.speak(utterance);
  });
}
