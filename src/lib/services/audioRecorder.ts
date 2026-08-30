// Voice message capture and WAV encoder matching parlezvous avatar architecture

let micStream: MediaStream | null = null;
let audioCtx: AudioContext | null = null;
let audioProcessor: ScriptProcessorNode | null = null;
let audioBuffers: Float32Array[] = [];
let recordingLength = 0;
let recognitionInstance: any = null;
let liveTranscription = '';
let isCurrentlyRecording = false;

export function isRecording(): boolean {
  return isCurrentlyRecording;
}

export async function startVoiceRecording(onTranscription?: (text: string) => void): Promise<void> {
  if (isCurrentlyRecording) {
    await stopVoiceRecording();
  }

  audioBuffers = [];
  recordingLength = 0;
  liveTranscription = '';

  try {
    micStream = await navigator.mediaDevices.getUserMedia({
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
    });

    audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 16000 });
    audioProcessor = audioCtx.createScriptProcessor(4096, 1, 1);
    const source = audioCtx.createMediaStreamSource(micStream);

    source.connect(audioProcessor);
    const gainNode = audioCtx.createGain();
    gainNode.gain.value = 0;
    audioProcessor.connect(gainNode);
    gainNode.connect(audioCtx.destination);

    audioProcessor.onaudioprocess = (e) => {
      if (isCurrentlyRecording) {
        const inputData = e.inputBuffer.getChannelData(0);
        audioBuffers.push(new Float32Array(inputData));
        recordingLength += inputData.length;
      }
    };

    // Parallel Web Speech recognition for instant visual transcript feedback
    const SpeechRec = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (SpeechRec) {
      try {
        recognitionInstance = new SpeechRec();
        recognitionInstance.continuous = true;
        recognitionInstance.interimResults = true;
        recognitionInstance.lang = 'en-US';

        recognitionInstance.onresult = (event: any) => {
          let text = '';
          for (let i = 0; i < event.results.length; i++) {
            text += event.results[i][0].transcript + ' ';
          }
          liveTranscription = text.trim();
          if (onTranscription) {
            onTranscription(liveTranscription);
          }
        };

        recognitionInstance.onerror = () => {
          // Gracefully continue audio capture even if speech recognition service is unavailable
        };

        recognitionInstance.start();
      } catch {
        // Ignore speech recognition error and continue recording PCM audio
      }
    }

    isCurrentlyRecording = true;
  } catch (err: any) {
    isCurrentlyRecording = false;
    throw new Error('Microphone access denied: ' + (err.message || err.name || 'Unknown error'));
  }
}

export async function stopVoiceRecording(): Promise<{ base64: string; wavBlob: Blob; transcription: string }> {
  isCurrentlyRecording = false;

  if (recognitionInstance) {
    try {
      recognitionInstance.stop();
    } catch {}
    recognitionInstance = null;
  }

  if (audioProcessor) {
    audioProcessor.disconnect();
    audioProcessor = null;
  }

  if (micStream) {
    micStream.getTracks().forEach((track) => track.stop());
    micStream = null;
  }

  const sampleRate = audioCtx?.sampleRate || 16000;
  if (audioCtx) {
    try {
      await audioCtx.close();
    } catch {}
    audioCtx = null;
  }

  if (recordingLength === 0) {
    return {
      base64: '',
      wavBlob: new Blob([], { type: 'audio/wav' }),
      transcription: liveTranscription.trim(),
    };
  }

  const wavBlob = exportWAV(audioBuffers, recordingLength, sampleRate);
  const base64 = await blobToBase64(wavBlob);

  const finalTranscription = liveTranscription.trim();

  // Reset buffers
  audioBuffers = [];
  recordingLength = 0;
  liveTranscription = '';

  return {
    base64,
    wavBlob,
    transcription: finalTranscription,
  };
}

function exportWAV(buffers: Float32Array[], length: number, sampleRate: number): Blob {
  const buffer = new Float32Array(length);
  let offset = 0;
  for (let i = 0; i < buffers.length; i++) {
    buffer.set(buffers[i], offset);
    offset += buffers[i].length;
  }
  const dataView = encodeWAV(buffer, sampleRate);
  return new Blob([dataView], { type: 'audio/wav' });
}

function encodeWAV(samples: Float32Array, sampleRate: number): DataView {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);

  // RIFF identifier
  writeString(view, 0, 'RIFF');
  // RIFF chunk length
  view.setUint32(4, 36 + samples.length * 2, true);
  // RIFF type
  writeString(view, 8, 'WAVE');
  // format chunk identifier
  writeString(view, 12, 'fmt ');
  // format chunk length
  view.setUint32(16, 16, true);
  // sample format (raw PCM)
  view.setUint16(20, 1, true);
  // channel count (mono)
  view.setUint16(22, 1, true);
  // sample rate
  view.setUint32(24, sampleRate, true);
  // byte rate (sample rate * block align)
  view.setUint32(28, sampleRate * 2, true);
  // block align (channel count * bytes per sample)
  view.setUint16(32, 2, true);
  // bits per sample
  view.setUint16(34, 16, true);
  // data chunk identifier
  writeString(view, 36, 'data');
  // data chunk length
  view.setUint32(40, samples.length * 2, true);

  // Write 16-bit PCM samples
  floatTo16BitPCM(view, 44, samples);
  return view;
}

function writeString(view: DataView, offset: number, string: string): void {
  for (let i = 0; i < string.length; i++) {
    view.setUint8(offset + i, string.charCodeAt(i));
  }
}

function floatTo16BitPCM(output: DataView, offset: number, input: Float32Array): void {
  for (let i = 0; i < input.length; i++, offset += 2) {
    const s = Math.max(-1, Math.min(1, input[i]));
    output.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const result = reader.result as string;
      const base64 = result.includes(',') ? result.split(',')[1] : result;
      resolve(base64);
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}
