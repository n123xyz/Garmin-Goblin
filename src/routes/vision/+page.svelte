<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { goblinState, loadGoblinProfile, setGoblinSpeech } from '$lib/state/goblin.svelte';
  import type { FoodIngestionResponse } from '$lib/types/garmin';
  import { Camera, Image as ImageIcon, Sparkles, Utensils, CheckCircle2, Mic, Square } from 'lucide-svelte';
  import toast from 'svelte-french-toast';
  import { saveFoodLog } from '$lib/state/journal.svelte';
  import { startVoiceRecording, stopVoiceRecording } from '$lib/services/audioRecorder';

  let selectedImageUri = $state<string | null>(null);
  let displayImageUrl = $state<string | null>(null);
  let userNotes = $state('');
  let isAnalyzing = $state(false);
  let analysisResult = $state<FoodIngestionResponse | null>(null);

  let isAutoLogged = $state(false);
  let loggedMealType = $state('lunch');

  // Voice Message Recording State
  let isRecordingVoice = $state(false);
  let recordedAudioBase64 = $state<string | null>(null);
  let liveTranscription = $state('');

  onMount(async () => {
    await loadGoblinProfile();
  });

  async function toggleVoiceRecording() {
    if (isRecordingVoice) {
      isRecordingVoice = false;
      try {
        const result = await stopVoiceRecording();
        if (result.base64) {
          recordedAudioBase64 = result.base64;
        }
        if (result.transcription) {
          userNotes = result.transcription;
        }
        toast.success('🎙️ Voice note recorded! Ready for Gemma analysis.');
      } catch (err: any) {
        toast.error('Failed to stop recording: ' + err.message);
      }
    } else {
      recordedAudioBase64 = null;
      liveTranscription = '';
      try {
        await startVoiceRecording((text) => {
          liveTranscription = text;
          userNotes = text;
        });
        isRecordingVoice = true;
        toast('🎙️ Recording voice note... describe what you ate or drank (e.g. coffee, 2 eggs)');
      } catch (err: any) {
        isRecordingVoice = false;
        toast.error(err.message || 'Microphone access failed');
      }
    }
  }

  async function takePhotoWithCamera() {
    try {
      const isAndroid = /Android/i.test(navigator.userAgent);
      if (isAndroid) {
        toast('Opening Camera...');
        const response: any = await invoke('plugin:litert|take_camera_photo');
        if (response && response.path) {
          selectedImageUri = response.path;
          displayImageUrl = convertFileSrc(response.path);
          analysisResult = null;
          isAutoLogged = false;
          toast.success('Photo captured!');
        }
      } else {
        // Desktop / browser camera capture fallback using file input capture
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = 'image/*';
        input.capture = 'environment';
        input.onchange = (e) => {
          const file = (e.target as HTMLInputElement).files?.[0];
          if (file) {
            displayImageUrl = URL.createObjectURL(file);
            const reader = new FileReader();
            reader.onloadend = () => {
              selectedImageUri = reader.result as string;
              analysisResult = null;
              isAutoLogged = false;
            };
            reader.readAsDataURL(file);
          }
        };
        input.click();
      }
    } catch (e) {
      if (e !== 'Camera capture cancelled by user') {
        toast.error('Camera capture failed: ' + e);
      }
    }
  }

  async function pickFromGallery() {
    try {
      const isAndroid = /Android/i.test(navigator.userAgent);
      if (isAndroid) {
        toast('Opening Gallery...');
        const response: any = await invoke('plugin:litert|pick_gallery_image');
        if (response && response.path) {
          selectedImageUri = response.path;
          displayImageUrl = convertFileSrc(response.path);
          analysisResult = null;
          isAutoLogged = false;
          toast.success('Image selected from gallery!');
        }
      } else {
        // Desktop / browser file picker
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = 'image/*';
        input.onchange = (e) => {
          const file = (e.target as HTMLInputElement).files?.[0];
          if (file) {
            displayImageUrl = URL.createObjectURL(file);
            const reader = new FileReader();
            reader.onloadend = () => {
              selectedImageUri = reader.result as string;
              analysisResult = null;
              isAutoLogged = false;
            };
            reader.readAsDataURL(file);
          }
        };
        input.click();
      }
    } catch (e) {
      if (e !== 'Image selection cancelled by user') {
        toast.error('Failed to select image: ' + e);
      }
    }
  }

  let analysisElapsedSeconds = $state(0);
  let timerInterval: any = null;

  async function runFoodAnalysis() {
    if ((!selectedImageUri && !userNotes.trim() && !recordedAudioBase64) || isAnalyzing) return;

    if (isRecordingVoice) {
      await toggleVoiceRecording();
    }

    isAnalyzing = true;
    isAutoLogged = false;
    analysisElapsedSeconds = 0;

    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => {
      analysisElapsedSeconds += 1;
    }, 1000);

    try {
      if (selectedImageUri) {
        toast('Gemma 4 Multimodal analyzing food photo & OCR on-device...');
      } else {
        toast('Gemma 4 calculating nutritional telemetry from meal notes...');
      }
      
      const result = await invoke<FoodIngestionResponse>('analyze_food_ingestion', {
        imageUri: selectedImageUri || null,
        userNotes: userNotes.trim() !== '' ? userNotes : null,
        audioBase64: recordedAudioBase64 || null,
      });

      analysisResult = result;
      setGoblinSpeech(result.goblin_comment, 6000);
      await loadGoblinProfile();

      // AUTO-LOG MEAL DIRECTLY TO HABITS & NUTRITION CALENDAR
      const now = new Date();
      const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
      const hour = now.getHours();
      const mealType = hour < 11 ? 'breakfast' : hour < 16 ? 'lunch' : hour < 21 ? 'dinner' : 'snack';
      loggedMealType = mealType;

      // Extract user liquid volume if specified (e.g. 250ml or 12oz or from result)
      let waterMl = result.water_ml || 0;
      if (waterMl === 0) {
        const mlMatch = userNotes.match(/(\d+)\s*ml/i);
        const ozMatch = userNotes.match(/(\d+(?:\.\d+)?)\s*oz/i);
        if (mlMatch) {
          waterMl = parseInt(mlMatch[1], 10);
        } else if (ozMatch) {
          waterMl = Math.round(parseFloat(ozMatch[1]) * 29.5735);
        } else if (/coffee|tea|water|juice|shake|beverage|smoothie/i.test(userNotes) || /coffee|tea|water|juice|shake|beverage|smoothie/i.test(result.food_title || '')) {
          waterMl = 250;
        }
      }

      const logSource = recordedAudioBase64 ? 'voice_message' : selectedImageUri ? (result.detected_type?.toLowerCase().includes('label') ? 'nutrition_label' : 'meal_photo') : 'manual';

      await saveFoodLog({
        date: dateStr,
        mealType,
        description: result.food_title || (userNotes.trim() ? userNotes.trim() : 'Meal Entry'),
        calories: result.estimated_calories || 0,
        carbsG: result.carbs_grams || 0,
        proteinG: result.protein_grams || 0,
        fatG: result.fat_grams || 0,
        fiberG: result.fiber_grams || 0,
        sugarG: result.sugar_grams || 0,
        sodiumMg: result.sodium_mg || 0,
        waterMl,
        notes: userNotes.trim() ? `${userNotes.trim()} — ${result.clinical_assessment}` : result.clinical_assessment,
        imagePath: selectedImageUri || undefined,
        sourceType: logSource,
      });

      isAutoLogged = true;
      toast.success('✓ Auto-Logged to Habits & Nutrition Calendar!');
    } catch (e: any) {
      toast.error('Analysis failed: ' + (e?.message || e));
    } finally {
      isAnalyzing = false;
      if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
      }
    }
  }

  function resetScanner() {
    if (isRecordingVoice) {
      stopVoiceRecording();
      isRecordingVoice = false;
    }
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    isAnalyzing = false;
    analysisElapsedSeconds = 0;
    selectedImageUri = null;
    displayImageUrl = null;
    analysisResult = null;
    recordedAudioBase64 = null;
    liveTranscription = '';
    isAutoLogged = false;
    userNotes = '';
  }
</script>

<div class="max-w-4xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Header Banner -->
  <div class="bg-gradient-to-r from-zinc-900 via-zinc-900 to-amber-950/30 border border-amber-500/25 rounded-3xl p-5 md:p-6 shadow-xl flex flex-col md:flex-row items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="w-12 h-12 rounded-2xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center text-2xl shrink-0 shadow-[0_0_15px_rgba(245,158,11,0.25)]">
        🥗
      </div>
      <div>
        <h1 class="text-xl md:text-2xl font-black text-zinc-100 flex items-center gap-2">
          Feed Gribble • Smart Food & Nutrition
        </h1>
        <p class="text-xs md:text-sm text-zinc-400 mt-0.5">
          Unified on-device multimodal scanner to feed Gribble plated meals, packaged food nutrition labels, beverages, and voice meal notes.
        </p>
      </div>
    </div>

    <!-- Feature Pills -->
    <div class="flex items-center gap-2 shrink-0">
      <span class="text-[11px] px-3 py-1.5 rounded-xl font-bold bg-zinc-950 text-amber-300 border border-amber-500/30 shadow-sm flex items-center gap-1.5">
        <Sparkles class="w-3.5 h-3.5 text-amber-400" />
        <span>Gemma 4 Multimodal</span>
      </span>
      <span class="text-[11px] px-3 py-1.5 rounded-xl font-bold bg-zinc-950 text-emerald-300 border border-emerald-500/30 shadow-sm flex items-center gap-1.5">
        <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
        <span>Auto-Sync Journal</span>
      </span>
    </div>
  </div>

  <!-- Camera / Gallery / Voice Input & Results Grid -->
  <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
    <!-- Input Box -->
    <div class="md:col-span-6 bg-zinc-900/90 border border-zinc-800 rounded-3xl p-4 shadow-xl flex flex-col min-h-[340px]">
      <div
        class="flex-1 rounded-2xl overflow-hidden bg-zinc-950 flex flex-col items-center justify-center border {displayImageUrl ? 'border-zinc-700' : 'border-zinc-800 border-dashed'} relative min-h-[220px]"
      >
        {#if displayImageUrl}
          <img src={displayImageUrl} alt="Food or Nutrition Label Target" class="w-full h-full object-cover" />
          <div class="absolute top-3 right-3 flex items-center gap-1.5">
            <button
              onclick={takePhotoWithCamera}
              class="w-9 h-9 bg-black/75 hover:bg-black text-white rounded-full flex items-center justify-center backdrop-blur-md transition cursor-pointer border border-zinc-700 shadow-md"
              title="Retake Photo with Camera"
            >
              <Camera class="w-4 h-4 text-emerald-400" />
            </button>
            <button
              onclick={pickFromGallery}
              class="w-9 h-9 bg-black/75 hover:bg-black text-white rounded-full flex items-center justify-center backdrop-blur-md transition cursor-pointer border border-zinc-700 shadow-md"
              title="Pick from Gallery"
            >
              <ImageIcon class="w-4 h-4 text-cyan-400" />
            </button>
            <button
              onclick={() => { displayImageUrl = null; selectedImageUri = null; }}
              class="w-9 h-9 bg-black/75 hover:bg-black text-rose-400 rounded-full flex items-center justify-center backdrop-blur-md transition cursor-pointer border border-zinc-700 shadow-md"
              title="Remove Photo"
            >
              ✕
            </button>
          </div>
        {:else if userNotes.trim() || recordedAudioBase64}
          <div class="text-center p-6 space-y-3 w-full">
            <div class="w-12 h-12 rounded-2xl bg-amber-500/20 text-amber-300 border border-amber-500/40 flex items-center justify-center text-2xl mx-auto shadow-[0_0_15px_rgba(245,158,11,0.2)]">
              {recordedAudioBase64 ? '🎙️' : '🍽️'}
            </div>
            <div class="space-y-1">
              <div class="flex items-center justify-center gap-2">
                <span class="text-xs px-2.5 py-0.5 rounded-full font-bold uppercase bg-amber-500/20 text-amber-300 border border-amber-500/30">
                  {recordedAudioBase64 ? 'Voice Audio Note' : 'Written Meal Log'}
                </span>
                <span class="text-[10px] text-zinc-400 font-mono">No Photo</span>
              </div>
              <p class="text-xs text-zinc-300 font-medium italic max-w-sm mx-auto line-clamp-3">
                "{userNotes || liveTranscription || 'Voice meal note attached'}"
              </p>
            </div>

            <!-- Optional Attach Photo Row -->
            <div class="flex items-center justify-center gap-2 pt-1">
              <button
                onclick={takePhotoWithCamera}
                class="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-[11px] font-semibold flex items-center gap-1.5 border border-zinc-700 transition cursor-pointer"
              >
                <Camera class="w-3.5 h-3.5 text-emerald-400" />
                <span>+ Attach Photo (Optional)</span>
              </button>
              <button
                onclick={pickFromGallery}
                class="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-[11px] font-semibold flex items-center gap-1.5 border border-zinc-700 transition cursor-pointer"
              >
                <ImageIcon class="w-3.5 h-3.5 text-cyan-400" />
                <span>Gallery</span>
              </button>
            </div>
          </div>
        {:else}
          <div class="text-center p-6 space-y-4 w-full">
            <div class="text-4xl">📸</div>
            <div class="space-y-1">
              <h3 class="text-sm md:text-base font-bold text-zinc-200">Capture Meal, Speak, or Type</h3>
              <p class="text-xs text-zinc-400">Photo is optional! Speak a voice note or type below to ingest meals.</p>
            </div>

            <!-- Action Buttons for Camera vs Gallery -->
            <div class="flex flex-col sm:flex-row items-center justify-center gap-2.5 pt-2 max-w-xs mx-auto">
              <button
                onclick={takePhotoWithCamera}
                class="w-full sm:w-auto px-4 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-black text-xs flex items-center justify-center gap-1.5 shadow-[0_0_12px_rgba(16,185,129,0.3)] transition-all cursor-pointer"
              >
                <Camera class="w-4 h-4" />
                <span>Take Photo</span>
              </button>

              <button
                onclick={pickFromGallery}
                class="w-full sm:w-auto px-4 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-bold text-xs flex items-center justify-center gap-1.5 border border-zinc-700 transition-all cursor-pointer"
              >
                <ImageIcon class="w-4 h-4 text-zinc-400" />
                <span>Gallery</span>
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Controls & Voice Row -->
      <div class="mt-4 space-y-3">
        <!-- Voice Record & Text Refinement Row -->
        <div class="space-y-2">
          <div class="flex items-center gap-2">
            <button
              onclick={toggleVoiceRecording}
              class="px-3.5 py-2 rounded-xl text-xs font-bold flex items-center gap-2 transition-all cursor-pointer border {isRecordingVoice ? 'bg-rose-600 hover:bg-rose-500 text-white border-rose-400 shadow-[0_0_15px_rgba(244,63,94,0.5)] animate-pulse' : recordedAudioBase64 ? 'bg-cyan-950 text-cyan-300 border-cyan-500/50 hover:bg-cyan-900/50' : 'bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border-zinc-700'}"
              title="Record Voice Meal Description"
            >
              {#if isRecordingVoice}
                <Square class="w-3.5 h-3.5 fill-current" />
                <span>Stop</span>
              {:else}
                <Mic class="w-3.5 h-3.5 {recordedAudioBase64 ? 'text-cyan-400' : 'text-emerald-400'}" />
                <span>{recordedAudioBase64 ? 'Re-record' : 'Voice'}</span>
              {/if}
            </button>

            <input
              type="text"
              placeholder={isRecordingVoice ? 'Listening to voice meal notes...' : 'Type or speak notes (e.g. 250ml coffee & 2 eggs, 1 tbsp olive oil)'}
              bind:value={userNotes}
              class="flex-1 bg-zinc-950 border {isRecordingVoice ? 'border-rose-500/50' : recordedAudioBase64 ? 'border-cyan-500/40' : 'border-zinc-800'} rounded-xl px-3 py-2 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-emerald-500 transition-colors"
            />
          </div>

          <!-- Live Voice Recording Banner -->
          {#if isRecordingVoice}
            <div class="flex items-center gap-2 px-3 py-2 rounded-xl bg-rose-950/40 border border-rose-500/30 text-rose-300 text-xs">
              <span class="w-2 h-2 rounded-full bg-rose-500 animate-ping"></span>
              <span class="font-medium">Recording voice meal notes... speak portions and items</span>
            </div>
          {:else if recordedAudioBase64}
            <div class="flex items-center justify-between px-3 py-1.5 rounded-xl bg-cyan-950/30 border border-cyan-500/30 text-cyan-300 text-[11px]">
              <div class="flex items-center gap-1.5">
                <span>🎙️</span>
                <span>Voice audio attached for multimodal parsing</span>
              </div>
              <button
                onclick={() => { recordedAudioBase64 = null; liveTranscription = ''; }}
                class="text-zinc-400 hover:text-zinc-200 text-[10px] underline cursor-pointer"
              >
                Remove Audio
              </button>
            </div>
          {/if}
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={runFoodAnalysis}
            disabled={(!selectedImageUri && !userNotes.trim() && !recordedAudioBase64) || isAnalyzing}
            class="flex-1 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 disabled:opacity-40 text-zinc-950 font-black text-xs md:text-sm flex items-center justify-center gap-2 shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-all cursor-pointer"
          >
            <Sparkles class="w-4 h-4 {isAnalyzing ? 'animate-spin' : ''}" />
            <span>
              {#if isAnalyzing}
                {#if selectedImageUri}
                  {#if analysisElapsedSeconds < 10}
                    Analyzing food photo ({analysisElapsedSeconds}s)...
                  {:else if analysisElapsedSeconds < 30}
                    Processing multimodal vision ({analysisElapsedSeconds}s)...
                  {:else}
                    Computing on-device inference ({analysisElapsedSeconds}s)...
                  {/if}
                {:else}
                  {#if analysisElapsedSeconds < 10}
                    Calculating meal nutrition ({analysisElapsedSeconds}s)...
                  {:else if analysisElapsedSeconds < 30}
                    Estimating macros & calories ({analysisElapsedSeconds}s)...
                  {:else}
                    Computing on-device inference ({analysisElapsedSeconds}s)...
                  {/if}
                {/if}
              {:else}
                Feed Gribble & Auto-Log Food
              {/if}
            </span>
          </button>

          {#if displayImageUrl || userNotes.trim() || recordedAudioBase64}
            <button
              onclick={resetScanner}
              class="px-3 py-2.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-semibold transition-colors cursor-pointer"
            >
              Reset
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Results Box -->
    <div class="md:col-span-6 bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 shadow-xl flex flex-col justify-between min-h-[340px]">
      {#if analysisResult}
        <div class="space-y-4">
          <!-- Header & Food Title -->
          <div class="flex items-start justify-between border-b border-zinc-800/80 pb-3">
            <div>
              <span class="text-xs px-2.5 py-0.5 rounded-full font-bold uppercase bg-amber-500/20 text-amber-300 border border-amber-500/30">
                {analysisResult.detected_type || 'Food / Meal'}
              </span>
              <h3 class="text-base font-bold text-zinc-100 mt-1.5">
                {analysisResult.food_title || 'Recognized Nutrition Target'}
              </h3>
              {#if analysisResult.serving_size}
                <span class="text-xs text-zinc-400 block mt-0.5">Portion: {analysisResult.serving_size}</span>
              {/if}
            </div>

            {#if analysisResult.health_score}
              <div class="text-right">
                <span class="text-2xl font-black text-emerald-400 font-mono">
                  {analysisResult.health_score}
                </span>
                <span class="text-[10px] text-zinc-500 block">/ 100 HEALTH</span>
              </div>
            {/if}
          </div>

          <!-- Nutritional Breakdown Primary Macros -->
          {#if analysisResult.estimated_calories !== undefined && analysisResult.estimated_calories !== null}
            <div class="grid grid-cols-4 gap-2 text-center font-mono">
              <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800">
                <span class="text-[10px] text-zinc-500 block">KCAL</span>
                <span class="text-xs font-bold text-amber-400">{analysisResult.estimated_calories}</span>
              </div>
              <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800">
                <span class="text-[10px] text-zinc-500 block">PROT</span>
                <span class="text-xs font-bold text-emerald-400">{analysisResult.protein_grams || 0}g</span>
              </div>
              <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800">
                <span class="text-[10px] text-zinc-500 block">CARB</span>
                <span class="text-xs font-bold text-cyan-400">{analysisResult.carbs_grams || 0}g</span>
              </div>
              <div class="bg-zinc-950/60 p-2 rounded-xl border border-zinc-800">
                <span class="text-[10px] text-zinc-500 block">FAT</span>
                <span class="text-xs font-bold text-rose-400">{analysisResult.fat_grams || 0}g</span>
              </div>
            </div>

            <!-- Secondary Nutrient Strip (Fiber, Sugar, Sodium, Water) -->
            <div class="flex items-center justify-between text-[11px] font-mono text-zinc-400 bg-zinc-950/40 px-3 py-1.5 rounded-xl border border-zinc-800/60">
              {#if analysisResult.fiber_grams}
                <span>Fiber: <strong class="text-zinc-200">{analysisResult.fiber_grams}g</strong></span>
              {/if}
              {#if analysisResult.sugar_grams}
                <span>Sugar: <strong class="text-zinc-200">{analysisResult.sugar_grams}g</strong></span>
              {/if}
              {#if analysisResult.sodium_mg}
                <span>Sodium: <strong class="text-zinc-200">{analysisResult.sodium_mg}mg</strong></span>
              {/if}
              {#if analysisResult.water_ml && analysisResult.water_ml > 0}
                <span class="text-cyan-400 font-bold">💧 {analysisResult.water_ml}ml</span>
              {/if}
            </div>
          {/if}

          <!-- Extracted Summary / Label Highlights -->
          {#if analysisResult.extracted_text}
            <div class="bg-zinc-950/50 p-2.5 rounded-xl border border-zinc-800/80 text-xs text-zinc-300">
              <span class="text-[10px] font-bold uppercase tracking-wider text-zinc-400 block mb-0.5">Identified Highlights:</span>
              <p class="text-zinc-200 leading-relaxed">{analysisResult.extracted_text}</p>
            </div>
          {/if}

          <!-- Clinical & Goblin Comments -->
          <div class="space-y-2">
            <p class="text-xs text-zinc-300 bg-zinc-900/60 p-2.5 rounded-xl border border-zinc-800">
              <span class="font-bold text-emerald-400 block mb-0.5">Clinical Telemetry:</span>
              {analysisResult.clinical_assessment}
            </p>

            <p class="text-xs text-amber-200 bg-amber-950/20 border border-amber-500/30 p-2.5 rounded-xl italic">
              <span class="font-bold text-amber-400 block mb-0.5">Gribble's Feast Reaction:</span>
              "{analysisResult.goblin_comment}"
            </p>
          </div>

          <!-- XP & Gold Loot Awarded -->
          <div class="flex items-center justify-between p-3 rounded-xl bg-gradient-to-r from-amber-500/15 to-emerald-500/15 border border-amber-500/30 text-xs font-bold">
            <span class="text-zinc-200">Loot Earned:</span>
            <div class="flex items-center gap-3">
              <span class="text-emerald-400">+{analysisResult.goblin_xp_awarded} XP</span>
              <span class="text-amber-400">🪙 +{analysisResult.goblin_gold_awarded} Gold</span>
            </div>
          </div>

          <!-- Auto-Logged Confirmation Banner -->
          {#if isAutoLogged}
            <div class="p-3.5 rounded-2xl bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-between gap-3">
              <div class="flex items-center gap-2.5">
                <CheckCircle2 class="w-5 h-5 text-emerald-400 shrink-0" />
                <div>
                  <span class="text-xs font-bold text-emerald-300 block">Auto-Logged to Habits & Nutrition</span>
                  <span class="text-[10px] text-zinc-400">Saved to today's {loggedMealType} entry</span>
                </div>
              </div>
              <a
                href="/journal"
                class="px-3 py-1.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-black text-xs transition-all shadow-sm shrink-0 flex items-center gap-1"
              >
                <span>View Calendar</span>
                <span>→</span>
              </a>
            </div>
          {/if}
        </div>
      {:else}
        <div class="h-full flex flex-col items-center justify-center text-center p-8 text-zinc-500 space-y-3">
          <Utensils class="w-12 h-12 text-zinc-700" />
          <h4 class="text-sm font-bold text-zinc-400">No Food Analyzed Yet</h4>
          <p class="text-xs max-w-xs text-zinc-500">
            Snap or select a photo of your meal, packaged nutrition label, or beverage, or record a voice meal note to analyze and auto-log nutritional telemetry.
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>

