<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    Calendar as CalendarIcon, ChevronLeft, ChevronRight, Utensils, 
    HeartHandshake, Plus, Trash2, Activity, Zap, Moon, Flame, 
    Droplets, Smile, Frown, Meh, AlertCircle, Sparkles, Check, 
    X, Clock, Tag, BookOpen, Compass, LayoutGrid, Columns, Mic, Square
  } from 'lucide-svelte';
  import { 
    journalState, loadCalendarMonth, setSelectedDate, 
    saveFoodLog, deleteFoodLog, saveEmotionalLog, deleteEmotionalLog,
    type FoodLog, type EmotionalLog, type CalendarDaySummary 
  } from '$lib/state/journal.svelte';
  import { goblinState, setGoblinSpeech } from '$lib/state/goblin.svelte';
  import { startVoiceRecording, stopVoiceRecording } from '$lib/services/audioRecorder';
  import { syncDeviceCalendar, calendarState } from '$lib/state/calendar.svelte';

  // --- View Mode & Modals ---
  let viewMode = $state<'week' | 'month'>('week');
  let showFoodModal = $state(false);
  let showEmotionModal = $state(false);
  let activeTab = $state<'all' | 'food' | 'emotion' | 'activities' | 'schedule'>('all');

  function getInitialMonday(): Date {
    const d = new Date();
    const day = d.getDay(); // 0 is Sunday
    const diff = d.getDate() - day + (day === 0 ? -6 : 1); // Monday start
    const monday = new Date(d.setDate(diff));
    monday.setHours(0, 0, 0, 0);
    return monday;
  }

  // --- Week State ---
  let currentWeekStart = $state<Date>(getInitialMonday());

  // --- Food Modal Form State ---
  let mealType = $state('lunch');
  let foodDescription = $state('');
  let foodCalories = $state<number | undefined>(undefined);
  let foodCarbs = $state<number | undefined>(undefined);
  let foodProtein = $state<number | undefined>(undefined);
  let foodFat = $state<number | undefined>(undefined);
  let foodWater = $state<number | undefined>(undefined);
  let foodNotes = $state('');
  let isFoodVoiceRecording = $state(false);

  // --- Emotion Modal Form State ---
  let selectedMood = $state('balanced');
  let energyLevel = $state(6);
  let motivationLevel = $state(6);
  let stressLevel = $state(3);
  let perceivedRecovery = $state(7);
  let selectedStressors = $state<string[]>([]);
  let customStressorInput = $state('');
  let emotionNotes = $state('');

  function getDeviceTimeOfDay(d: Date = new Date()): string {
    const hour = d.getHours();
    if (hour >= 5 && hour < 12) return 'morning';
    if (hour >= 12 && hour < 17) return 'afternoon';
    if (hour >= 17 && hour < 21) return 'evening';
    return 'night';
  }

  const MOOD_OPTIONS = [
    { key: 'energized', label: 'Energized', icon: '⚡', color: 'text-amber-400 border-amber-500/40 bg-amber-500/10' },
    { key: 'calm', label: 'Calm', icon: '🧘', color: 'text-emerald-400 border-emerald-500/40 bg-emerald-500/10' },
    { key: 'focused', label: 'Focused', icon: '🎯', color: 'text-cyan-400 border-cyan-500/40 bg-cyan-500/10' },
    { key: 'balanced', label: 'Balanced', icon: '⚖️', color: 'text-indigo-400 border-indigo-500/40 bg-indigo-500/10' },
    { key: 'fatigued', label: 'Fatigued', icon: '🥱', color: 'text-orange-400 border-orange-500/40 bg-orange-500/10' },
    { key: 'stressed', label: 'Stressed', icon: '💥', color: 'text-rose-400 border-rose-500/40 bg-rose-500/10' },
    { key: 'anxious', label: 'Anxious', icon: '🌀', color: 'text-purple-400 border-purple-500/40 bg-purple-500/10' },
    { key: 'low', label: 'Low', icon: '🌧️', color: 'text-zinc-400 border-zinc-500/40 bg-zinc-500/10' },
  ];

  const COMMON_STRESSORS = [
    'Work Deadline', 'Poor Sleep', 'Muscle Soreness', 'Training Fatigue',
    'Nutrition / Dehydration', 'Interpersonal Stress', 'Travel / Commute', 'Illness'
  ];

  const MEAL_TYPES = [
    { key: 'breakfast', label: 'Breakfast', icon: '🍳' },
    { key: 'lunch', label: 'Lunch', icon: '🥗' },
    { key: 'dinner', label: 'Dinner', icon: '🍲' },
    { key: 'snack', label: 'Snack', icon: '🍎' },
    { key: 'pre_workout', label: 'Pre-Workout', icon: '⚡' },
    { key: 'post_workout', label: 'Post-Workout', icon: '💪' },
  ];

  function formatEventTime(isoString: string): string {
    try {
      const d = new Date(isoString);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return isoString;
    }
  }

  onMount(async () => {
    await syncDeviceCalendar();
    await loadCalendarMonth();
    if (journalState.calendarData && journalState.calendarData.days.length > 0) {
      const today = journalState.calendarData.days.find((d) => d.isToday);
      if (today) {
        setSelectedDate(today.date);
      }
    }
  });

  function getWeekDateStrings(startDate: Date): string[] {
    const dates: string[] = [];
    for (let i = 0; i < 7; i++) {
      const d = new Date(startDate);
      d.setDate(startDate.getDate() + i);
      const yyyy = d.getFullYear();
      const mm = String(d.getMonth() + 1).padStart(2, '0');
      const dd = String(d.getDate()).padStart(2, '0');
      dates.push(`${yyyy}-${mm}-${dd}`);
    }
    return dates;
  }

  let weekDateStrings = $derived(getWeekDateStrings(currentWeekStart));

  let weekDaysSummaries = $derived(() => {
    const list: CalendarDaySummary[] = [];
    const allDays = journalState.calendarData?.days || [];
    
    for (const dateStr of weekDateStrings) {
      const found = allDays.find((d) => d.date === dateStr);
      if (found) {
        list.push(found);
      } else {
        const parts = dateStr.split('-');
        list.push({
          date: dateStr,
          dayOfMonth: parseInt(parts[2], 10),
          isCurrentMonth: true,
          isToday: isDateToday(dateStr),
          sleepScore: 0,
          steps: 0,
          restingHr: 0,
          hrvStatus: 'UNKNOWN',
          hrvRmssd: 0,
          stressLevel: 0,
          totalFoodCalories: 0,
          totalActiveCalories: 0,
          foodCount: 0,
          activityCount: 0,
          latestMood: undefined,
          avgEmotionalStress: undefined,
          foods: [],
          emotions: [],
          activities: [],
          events: [],
        });
      }
    }
    return list;
  });

  function isDateToday(dateStr: string): boolean {
    const now = new Date();
    const todayStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    return dateStr === todayStr;
  }

  function formatWeekRange(startDate: Date): string {
    const end = new Date(startDate);
    end.setDate(startDate.getDate() + 6);
    const sMonth = startDate.toLocaleDateString([], { month: 'short' });
    const eMonth = end.toLocaleDateString([], { month: 'short' });
    if (sMonth === eMonth) {
      return `${sMonth} ${startDate.getDate()} - ${end.getDate()}, ${startDate.getFullYear()}`;
    }
    return `${sMonth} ${startDate.getDate()} - ${eMonth} ${end.getDate()}, ${end.getFullYear()}`;
  }

  function handlePrevWeek() {
    const prev = new Date(currentWeekStart);
    prev.setDate(currentWeekStart.getDate() - 7);
    currentWeekStart = prev;
    ensureMonthLoaded(prev);
  }

  function handleNextWeek() {
    const next = new Date(currentWeekStart);
    next.setDate(currentWeekStart.getDate() + 7);
    currentWeekStart = next;
    ensureMonthLoaded(next);
  }

  function handleToday() {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    const monday = new Date(d.setDate(diff));
    monday.setHours(0, 0, 0, 0);
    currentWeekStart = monday;
    
    const now = new Date();
    const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    setSelectedDate(dateStr);
    ensureMonthLoaded(now);
  }

  function ensureMonthLoaded(date: Date) {
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    if (journalState.currentYear !== year || journalState.currentMonth !== month) {
      loadCalendarMonth(year, month);
    }
  }

  function handlePrevMonth() {
    let year = journalState.currentYear;
    let month = journalState.currentMonth - 1;
    if (month < 1) {
      month = 12;
      year -= 1;
    }
    loadCalendarMonth(year, month);
  }

  function handleNextMonth() {
    let year = journalState.currentYear;
    let month = journalState.currentMonth + 1;
    if (month > 12) {
      month = 1;
      year += 1;
    }
    loadCalendarMonth(year, month);
  }

  function addCustomStressor() {
    const trimmed = customStressorInput.trim();
    if (trimmed && !selectedStressors.includes(trimmed)) {
      selectedStressors = [...selectedStressors, trimmed];
    }
    customStressorInput = '';
  }

  function removeStressor(item: string) {
    selectedStressors = selectedStressors.filter((s) => s !== item);
  }

  function toggleStressor(item: string) {
    if (selectedStressors.includes(item)) {
      selectedStressors = selectedStressors.filter((s) => s !== item);
    } else {
      selectedStressors = [...selectedStressors, item];
    }
  }

  function handleStressorKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      addCustomStressor();
    }
  }

  async function toggleFoodVoiceRecording() {
    if (isFoodVoiceRecording) {
      try {
        const res = await stopVoiceRecording();
        isFoodVoiceRecording = false;
        if (res.transcription) {
          foodDescription = res.transcription;

          // Smart regex extraction from spoken audio
          const calMatch = res.transcription.match(/(\d+)\s*(?:cal|calories|kcal)/i);
          if (calMatch) foodCalories = parseInt(calMatch[1], 10);

          const mlMatch = res.transcription.match(/(\d+)\s*ml/i);
          if (mlMatch) foodWater = parseInt(mlMatch[1], 10);

          const protMatch = res.transcription.match(/(\d+)\s*(?:g|grams)?\s*(?:of\s*)?protein/i);
          if (protMatch) foodProtein = parseInt(protMatch[1], 10);
        }
      } catch (err: any) {
        isFoodVoiceRecording = false;
      }
    } else {
      try {
        await startVoiceRecording((text) => {
          foodDescription = text;
        });
        isFoodVoiceRecording = true;
      } catch (err: any) {
        isFoodVoiceRecording = false;
      }
    }
  }

  async function submitFoodLog() {
    if (isFoodVoiceRecording) {
      await toggleFoodVoiceRecording();
    }
    if (!foodDescription.trim()) return;
    await saveFoodLog({
      date: journalState.selectedDate,
      mealType,
      description: foodDescription.trim(),
      calories: foodCalories,
      carbsG: foodCarbs,
      proteinG: foodProtein,
      fatG: foodFat,
      waterMl: foodWater,
      notes: foodNotes.trim() || undefined,
    });
    setGoblinSpeech(`Mmm! Nutrition logged! Keep feeding the beast properly!`);
    foodDescription = '';
    foodNotes = '';
    showFoodModal = false;
  }

  async function submitEmotionLog() {
    if (customStressorInput.trim()) {
      addCustomStressor();
    }
    const now = new Date();
    await saveEmotionalLog({
      date: journalState.selectedDate,
      timeOfDay: getDeviceTimeOfDay(now),
      mood: selectedMood,
      energyLevel,
      motivationLevel,
      stressLevel,
      perceivedRecovery,
      stressors: selectedStressors,
      notes: emotionNotes.trim() || undefined,
    });
    setGoblinSpeech(`Checked in! Emotional awareness is true mental armor, mortal!`);
    emotionNotes = '';
    customStressorInput = '';
    selectedStressors = [];
    showEmotionModal = false;
  }

  function formatDayName(dateStr: string): string {
    try {
      const dt = new Date(dateStr + 'T00:00:00');
      return dt.toLocaleDateString([], { weekday: 'short' });
    } catch {
      return '';
    }
  }

  function getMoodBadge(mood: string) {
    return MOOD_OPTIONS.find((m) => m.key === mood) || MOOD_OPTIONS[3];
  }

  let firstDayOfWeek = $derived(() => {
    if (!journalState.calendarData || journalState.calendarData.days.length === 0) return 0;
    const firstDate = new Date(`${journalState.calendarData.year}-${String(journalState.calendarData.month).padStart(2, '0')}-01T00:00:00`);
    return firstDate.getDay();
  });
</script>

<div class="max-w-6xl mx-auto p-4 md:p-6 w-full space-y-6">
  <!-- Top Navigation & View Switcher -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800 pb-4">
    <div>
      <div class="flex items-center gap-2">
        <span class="text-2xl md:text-3xl">📅</span>
        <h1 class="text-2xl md:text-3xl font-black text-zinc-100">Habits & Journal</h1>
      </div>
      <p class="text-xs md:text-sm text-zinc-400 mt-1">
        Weekly nutrition logs, emotional status check-ins, and Garmin activity telemetry.
      </p>
    </div>

    <!-- View Mode Switcher + Action Buttons -->
    <div class="flex flex-wrap items-center gap-2">
      <!-- Week vs Month Toggle -->
      <div class="flex items-center bg-zinc-900 border border-zinc-800 rounded-2xl p-1 shadow-sm">
        <button
          onclick={() => viewMode = 'week'}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {viewMode === 'week' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <Columns class="w-3.5 h-3.5" />
          <span>Week</span>
        </button>
        <button
          onclick={() => viewMode = 'month'}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {viewMode === 'month' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <LayoutGrid class="w-3.5 h-3.5" />
          <span>Month</span>
        </button>
      </div>

      <!-- Quick Action Buttons -->
      <button 
        onclick={() => showFoodModal = true}
        class="flex items-center gap-1.5 px-3 py-2 rounded-2xl bg-amber-500/15 hover:bg-amber-500/25 border border-amber-500/30 text-amber-300 text-xs font-bold transition-all shadow-sm active:scale-95 cursor-pointer"
      >
        <Utensils class="w-3.5 h-3.5" />
        <span>+ Log Food</span>
      </button>

      <button 
        onclick={() => showEmotionModal = true}
        class="flex items-center gap-1.5 px-3 py-2 rounded-2xl bg-cyan-500/15 hover:bg-cyan-500/25 border border-cyan-500/30 text-cyan-300 text-xs font-bold transition-all shadow-sm active:scale-95 cursor-pointer"
      >
        <HeartHandshake class="w-3.5 h-3.5" />
        <span>+ Log Mood</span>
      </button>
    </div>
  </div>

  <!-- Weekly View / Monthly View Header Strip -->
  <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-4 md:p-6 shadow-xl space-y-4">
    <div class="flex items-center justify-between border-b border-zinc-800/80 pb-3 flex-wrap gap-2">
      <!-- Week or Month Pager -->
      <div class="flex items-center gap-2">
        {#if viewMode === 'week'}
          <button 
            onclick={handlePrevWeek}
            class="p-1.5 hover:bg-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 transition-colors border border-zinc-800 cursor-pointer"
            aria-label="Previous Week"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <span class="text-xs md:text-sm font-black font-mono text-zinc-100 px-2">
            {formatWeekRange(currentWeekStart)}
          </span>
          <button 
            onclick={handleNextWeek}
            class="p-1.5 hover:bg-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 transition-colors border border-zinc-800 cursor-pointer"
            aria-label="Next Week"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
          <button
            onclick={handleToday}
            class="ml-1 text-[11px] px-2.5 py-1 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-300 font-bold transition-all border border-zinc-700 cursor-pointer"
          >
            Today
          </button>
        {:else}
          <button 
            onclick={handlePrevMonth}
            class="p-1.5 hover:bg-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 transition-colors border border-zinc-800 cursor-pointer"
            aria-label="Previous Month"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <span class="text-xs md:text-sm font-black font-mono text-zinc-100 px-2">
            {journalState.calendarData?.monthName || 'Month'} {journalState.currentYear}
          </span>
          <button 
            onclick={handleNextMonth}
            class="p-1.5 hover:bg-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 transition-colors border border-zinc-800 cursor-pointer"
            aria-label="Next Month"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        {/if}
      </div>

      <!-- Legend -->
      <div class="flex items-center gap-3 text-[10px] text-zinc-400 font-medium">
        <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-amber-400"></span> Food</span>
        <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-cyan-400"></span> Mood</span>
        <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-emerald-400"></span> Workout</span>
      </div>
    </div>

    <!-- WEEK VIEW (Mobile & Responsive Strip) -->
    {#if viewMode === 'week'}
      <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2.5">
        {#each weekDaysSummaries() as day}
          {@const isSelected = day.date === journalState.selectedDate}
          {@const moodBadge = day.latestMood ? getMoodBadge(day.latestMood) : null}
          <button
            onclick={() => setSelectedDate(day.date)}
            class="p-3 rounded-2xl border text-left flex flex-col justify-between transition-all duration-200 min-h-[140px] cursor-pointer
              {isSelected 
                ? 'bg-emerald-950/40 border-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.25)] ring-2 ring-emerald-500' 
                : day.isToday 
                  ? 'bg-zinc-800/90 border-emerald-500/60 hover:border-zinc-700' 
                  : 'bg-zinc-950/70 border-zinc-800/80 hover:bg-zinc-800/50 hover:border-zinc-700'}"
          >
            <!-- Day Header -->
            <div class="flex items-center justify-between w-full border-b border-zinc-800/60 pb-1.5">
              <div>
                <span class="text-[10px] font-bold uppercase tracking-wider block text-zinc-400">
                  {formatDayName(day.date)}
                </span>
                <span class="text-sm font-black font-mono {day.isToday ? 'text-emerald-400' : isSelected ? 'text-zinc-100' : 'text-zinc-300'}">
                  {day.dayOfMonth}
                </span>
              </div>

              {#if day.isToday}
                <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/40">
                  TODAY
                </span>
              {/if}
            </div>

            <!-- Badges Section -->
            <div class="space-y-1.5 my-2 w-full">
              <!-- Workout Pill -->
              {#if day.activityCount > 0}
                <div class="text-[10px] font-bold px-2 py-1 rounded-xl bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 truncate flex items-center gap-1">
                  <span>🏃</span>
                  <span class="truncate">{day.activities[0].title}</span>
                </div>
              {/if}

              <!-- Food Intake Pill -->
              {#if day.foodCount > 0}
                <div class="text-[10px] font-bold px-2 py-1 rounded-xl bg-amber-500/20 text-amber-300 border border-amber-500/30 truncate flex items-center gap-1">
                  <span>🥗</span>
                  <span>{day.totalFoodCalories} kcal</span>
                </div>
              {/if}

              <!-- Mood Pill -->
              {#if moodBadge}
                <div class="text-[10px] font-bold px-2 py-1 rounded-xl {moodBadge.color} truncate flex items-center gap-1">
                  <span>{moodBadge.icon}</span>
                  <span class="capitalize truncate">{moodBadge.label}</span>
                </div>
              {/if}
            </div>

            <!-- Mini Biometrics Footer -->
            <div class="flex items-center justify-between text-[10px] font-mono text-zinc-400 pt-1.5 border-t border-zinc-800/60">
              {#if day.sleepScore > 0}
                <span class="text-cyan-400 font-bold">🌙 {day.sleepScore}</span>
              {:else}
                <span class="text-zinc-600">--</span>
              {/if}
              {#if day.steps > 0}
                <span class="text-zinc-300 font-bold">👣 {(day.steps / 1000).toFixed(1)}k</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <!-- MONTH VIEW (Full Matrix) -->
      <div class="grid grid-cols-7 gap-1.5 text-center text-[11px] font-bold text-zinc-400 uppercase tracking-wider py-1">
        <span>Sun</span>
        <span>Mon</span>
        <span>Tue</span>
        <span>Wed</span>
        <span>Thu</span>
        <span>Fri</span>
        <span>Sat</span>
      </div>

      <div class="grid grid-cols-7 gap-1.5 sm:gap-2">
        <!-- Empty leading cells for month offset -->
        {#each Array(firstDayOfWeek()) as _}
          <div class="min-h-[75px] sm:min-h-[90px] rounded-2xl bg-zinc-950/20 border border-transparent opacity-30"></div>
        {/each}

        <!-- Month Days -->
        {#if journalState.calendarData}
          {#each journalState.calendarData.days as day}
            {@const isSelected = day.date === journalState.selectedDate}
            {@const moodBadge = day.latestMood ? getMoodBadge(day.latestMood) : null}
            <button
              onclick={() => setSelectedDate(day.date)}
              class="min-h-[75px] sm:min-h-[95px] p-2 rounded-2xl border text-left flex flex-col justify-between transition-all duration-200 relative group cursor-pointer
                {isSelected 
                  ? 'bg-emerald-950/30 border-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.2)] ring-1 ring-emerald-500' 
                  : day.isToday 
                    ? 'bg-zinc-800/80 border-emerald-500/50 hover:border-zinc-700' 
                    : 'bg-zinc-950/70 border-zinc-800/80 hover:bg-zinc-800/50 hover:border-zinc-700'}"
            >
              <div class="flex items-center justify-between w-full">
                <span class="text-xs font-black font-mono {day.isToday ? 'text-emerald-400' : isSelected ? 'text-zinc-100' : 'text-zinc-400'}">
                  {day.dayOfMonth}
                </span>

                {#if day.isToday}
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                {/if}
              </div>

              <div class="space-y-1 my-1 w-full overflow-hidden">
                {#if day.activityCount > 0}
                  <div class="text-[9px] font-bold px-1.5 py-0.5 rounded-md bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 truncate flex items-center gap-1">
                    <span>🏃</span>
                    <span class="truncate">{day.activities[0].title}</span>
                  </div>
                {/if}

                {#if day.foodCount > 0}
                  <div class="text-[9px] font-bold px-1.5 py-0.5 rounded-md bg-amber-500/20 text-amber-300 border border-amber-500/30 truncate flex items-center gap-1">
                    <span>🥗</span>
                    <span>{day.totalFoodCalories} kcal</span>
                  </div>
                {/if}

                {#if moodBadge}
                  <div class="text-[9px] font-bold px-1.5 py-0.5 rounded-md {moodBadge.color} truncate flex items-center gap-1">
                    <span>{moodBadge.icon}</span>
                    <span class="capitalize truncate">{moodBadge.label}</span>
                  </div>
                {/if}
              </div>

              <div class="flex items-center justify-between text-[9px] font-mono text-zinc-400 pt-0.5 border-t border-zinc-800/50">
                {#if day.sleepScore > 0}
                  <span class="text-cyan-400">🌙 {day.sleepScore}</span>
                {:else}
                  <span></span>
                {/if}
                {#if day.steps > 0}
                  <span class="text-zinc-400 font-bold">👣 {(day.steps / 1000).toFixed(1)}k</span>
                {/if}
              </div>
            </button>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- Selected Day Detailed Timeline & Metrics -->
  {#if journalState.selectedDaySummary}
    {@const day = journalState.selectedDaySummary}
    <div class="bg-zinc-900/90 border border-zinc-800 rounded-3xl p-5 md:p-6 shadow-xl space-y-5">
      <!-- Day Title & Summary Card -->
      <div class="border-b border-zinc-800/80 pb-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <span class="text-[10px] font-bold uppercase tracking-wider text-emerald-400 block">Active Date Telemetry</span>
          <h2 class="text-xl md:text-2xl font-black text-zinc-100 font-mono flex items-center gap-2">
            <span>{day.date}</span>
            {#if isDateToday(day.date)}
              <span class="text-xs px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 font-bold">TODAY</span>
            {/if}
          </h2>
        </div>

        <div class="flex items-center gap-2">
          <button 
            onclick={() => showFoodModal = true}
            class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-amber-500/20 text-amber-300 hover:bg-amber-500/30 border border-amber-500/40 text-xs font-bold transition-all shadow-sm active:scale-95 cursor-pointer"
          >
            <Utensils class="w-4 h-4" />
            <span>Log Meal</span>
          </button>
          <button 
            onclick={() => showEmotionModal = true}
            class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-cyan-500/20 text-cyan-300 hover:bg-cyan-500/30 border border-cyan-500/40 text-xs font-bold transition-all shadow-sm active:scale-95 cursor-pointer"
          >
            <HeartHandshake class="w-4 h-4" />
            <span>Check In</span>
          </button>
        </div>
      </div>

      <!-- Key Metrics Highlights -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-center">
        <div class="bg-zinc-950/80 p-3.5 rounded-2xl border border-zinc-800/80">
          <span class="text-[10px] uppercase font-bold text-zinc-400 block">Food Intake</span>
          <span class="text-base md:text-lg font-black font-mono text-amber-400">{day.totalFoodCalories} kcal</span>
          <span class="text-[10px] text-zinc-400 block mt-0.5">{day.foodCount} meals logged</span>
        </div>

        <div class="bg-zinc-950/80 p-3.5 rounded-2xl border border-zinc-800/80">
          <span class="text-[10px] uppercase font-bold text-zinc-400 block">Active Output</span>
          <span class="text-base md:text-lg font-black font-mono text-emerald-400">{day.totalActiveCalories} kcal</span>
          <span class="text-[10px] text-zinc-400 block mt-0.5">{day.activityCount} workouts</span>
        </div>

        <div class="bg-zinc-950/80 p-3.5 rounded-2xl border border-zinc-800/80">
          <span class="text-[10px] uppercase font-bold text-zinc-400 block">Sleep Score</span>
          <span class="text-base md:text-lg font-black font-mono text-cyan-400">{day.sleepScore > 0 ? `${day.sleepScore}/100` : '--'}</span>
          <span class="text-[10px] text-zinc-400 block mt-0.5">{day.restingHr > 0 ? `${day.restingHr} bpm rHR` : 'Garmin Sync'}</span>
        </div>

        <div class="bg-zinc-950/80 p-3.5 rounded-2xl border border-zinc-800/80">
          <span class="text-[10px] uppercase font-bold text-zinc-400 block">Subjective Stress</span>
          <span class="text-base md:text-lg font-black font-mono text-purple-400">{day.avgEmotionalStress ? `${day.avgEmotionalStress}/10` : '--'}</span>
          <span class="text-[10px] text-zinc-400 block mt-0.5">{day.emotions.length} check-ins</span>
        </div>
      </div>

      <!-- Section Filter Tabs -->
      <div class="flex items-center gap-1.5 bg-zinc-950 p-1.5 rounded-2xl border border-zinc-800/80 text-xs font-bold overflow-x-auto">
        <button 
          onclick={() => activeTab = 'all'} 
          class="flex-1 min-w-[70px] py-2 rounded-xl transition-all cursor-pointer {activeTab === 'all' ? 'bg-zinc-800 text-zinc-100 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          All
        </button>
        <button 
          onclick={() => activeTab = 'food'} 
          class="flex-1 min-w-[90px] py-2 rounded-xl transition-all cursor-pointer {activeTab === 'food' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Food ({day.foods.length})
        </button>
        <button 
          onclick={() => activeTab = 'emotion'} 
          class="flex-1 min-w-[90px] py-2 rounded-xl transition-all cursor-pointer {activeTab === 'emotion' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Mood ({day.emotions.length})
        </button>
        <button 
          onclick={() => activeTab = 'activities'} 
          class="flex-1 min-w-[90px] py-2 rounded-xl transition-all cursor-pointer {activeTab === 'activities' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Sports ({day.activities.length})
        </button>
        <button 
          onclick={() => activeTab = 'schedule'} 
          class="flex-1 min-w-[90px] py-2 rounded-xl transition-all cursor-pointer {activeTab === 'schedule' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Schedule ({day.events?.length || 0})
        </button>
      </div>

      <!-- Timeline Content Feed -->
      <div class="space-y-4">
        <!-- Food Logs Section -->
        {#if activeTab === 'all' || activeTab === 'food'}
          {#if day.foods.length > 0}
            <div class="space-y-2.5">
              <span class="text-xs font-bold text-amber-400 flex items-center gap-1.5">
                <Utensils class="w-4 h-4" /> Nutrition & Meals ({day.foods.length})
              </span>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each day.foods as food}
                  <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 space-y-2 relative group shadow-sm">
                    <div class="flex items-start justify-between gap-2">
                      <div>
                        <span class="text-[10px] font-bold uppercase tracking-wider text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded-md border border-amber-500/20">
                          {food.mealType}
                        </span>
                        <h4 class="text-sm font-bold text-zinc-100 mt-1">{food.description}</h4>
                      </div>
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-black font-mono text-amber-400">{food.calories} kcal</span>
                        <button 
                          onclick={() => deleteFoodLog(food.id, day.date)}
                          class="p-1.5 text-zinc-400 hover:text-rose-400 transition-colors rounded-lg hover:bg-zinc-900 cursor-pointer"
                          title="Delete entry"
                        >
                          <Trash2 class="w-4 h-4" />
                        </button>
                      </div>
                    </div>

                    <!-- Macros Bar -->
                    <div class="flex items-center gap-3 text-xs font-mono text-zinc-400 pt-2 border-t border-zinc-800/50">
                      <span>C: <strong class="text-zinc-200">{food.carbsG}g</strong></span>
                      <span>P: <strong class="text-zinc-200">{food.proteinG}g</strong></span>
                      <span>F: <strong class="text-zinc-200">{food.fatG}g</strong></span>
                      {#if food.waterMl > 0}
                        <span class="text-cyan-400 font-bold">💧 {food.waterMl}ml</span>
                      {/if}
                    </div>

                    {#if food.notes}
                      <p class="text-xs text-zinc-400 italic bg-zinc-900/60 p-2 rounded-xl">"{food.notes}"</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {:else if activeTab === 'food'}
            <div class="p-8 text-center text-zinc-400 text-xs border border-dashed border-zinc-800 rounded-3xl space-y-2">
              <Utensils class="w-8 h-8 text-zinc-600 mx-auto" />
              <p>No food logs recorded for this day. Tap <strong>"+ Log Meal"</strong> to track your nutrition!</p>
            </div>
          {/if}
        {/if}

        <!-- Emotional Check-ins Section -->
        {#if activeTab === 'all' || activeTab === 'emotion'}
          {#if day.emotions.length > 0}
            <div class="space-y-2.5">
              <span class="text-xs font-bold text-cyan-400 flex items-center gap-1.5">
                <HeartHandshake class="w-4 h-4" /> Subjective Emotional Check-ins ({day.emotions.length})
              </span>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each day.emotions as em}
                  {@const badge = getMoodBadge(em.mood)}
                  <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 space-y-3 relative group shadow-sm">
                    <div class="flex items-start justify-between gap-2">
                      <div class="flex items-center gap-2">
                        <span class="text-xl">{badge.icon}</span>
                        <div>
                          <span class="text-xs font-bold text-zinc-100 capitalize">{badge.label}</span>
                          <span class="text-[10px] text-zinc-400 block uppercase font-mono">{em.timeOfDay}</span>
                        </div>
                      </div>
                      <button 
                        onclick={() => deleteEmotionalLog(em.id, day.date)}
                        class="p-1.5 text-zinc-400 hover:text-rose-400 transition-colors rounded-lg hover:bg-zinc-900 cursor-pointer"
                        title="Delete entry"
                      >
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>

                    <!-- 1 to 10 Levels Grid -->
                    <div class="grid grid-cols-4 gap-1.5 text-center text-xs font-mono">
                      <div class="bg-zinc-900 p-1.5 rounded-xl border border-zinc-800">
                        <span class="text-[9px] uppercase text-zinc-400 block">Energy</span>
                        <span class="font-bold text-amber-400">{em.energyLevel}/10</span>
                      </div>
                      <div class="bg-zinc-900 p-1.5 rounded-xl border border-zinc-800">
                        <span class="text-[9px] uppercase text-zinc-400 block">Drive</span>
                        <span class="font-bold text-cyan-400">{em.motivationLevel}/10</span>
                      </div>
                      <div class="bg-zinc-900 p-1.5 rounded-xl border border-zinc-800">
                        <span class="text-[9px] uppercase text-zinc-400 block">Stress</span>
                        <span class="font-bold text-rose-400">{em.stressLevel}/10</span>
                      </div>
                      <div class="bg-zinc-900 p-1.5 rounded-xl border border-zinc-800">
                        <span class="text-[9px] uppercase text-zinc-400 block">Recovery</span>
                        <span class="font-bold text-emerald-400">{em.perceivedRecovery}/10</span>
                      </div>
                    </div>

                    <!-- Stressors -->
                    {#if em.stressors && em.stressors.length > 0}
                      <div class="flex flex-wrap gap-1">
                        {#each em.stressors as st}
                          <span class="text-[9px] px-2 py-0.5 rounded-md bg-rose-500/10 text-rose-300 border border-rose-500/20 font-medium">
                            {st}
                          </span>
                        {/each}
                      </div>
                    {/if}

                    {#if em.notes}
                      <p class="text-xs text-zinc-400 italic bg-zinc-900/60 p-2.5 rounded-xl">"{em.notes}"</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {:else if activeTab === 'emotion'}
            <div class="p-8 text-center text-zinc-400 text-xs border border-dashed border-zinc-800 rounded-3xl space-y-2">
              <HeartHandshake class="w-8 h-8 text-zinc-600 mx-auto" />
              <p>No emotional check-ins recorded for this day. Tap <strong>"+ Check In"</strong> to log your mood and energy!</p>
            </div>
          {/if}
        {/if}

        <!-- Garmin Activities Section -->
        {#if activeTab === 'all' || activeTab === 'activities'}
          {#if day.activities.length > 0}
            <div class="space-y-2.5">
              <span class="text-xs font-bold text-emerald-400 flex items-center gap-1.5">
                <Activity class="w-4 h-4" /> Garmin Watch Workouts ({day.activities.length})
              </span>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each day.activities as act}
                  <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 space-y-2 shadow-sm">
                    <div class="flex items-start justify-between gap-2">
                      <div>
                        <span class="text-[10px] font-bold uppercase tracking-wider text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
                          {act.sport}
                        </span>
                        <h4 class="text-sm font-bold text-zinc-100 mt-1">{act.title}</h4>
                      </div>
                      <span class="text-sm font-black font-mono text-emerald-400">{act.calories} kcal</span>
                    </div>

                    <div class="flex items-center gap-4 text-xs font-mono text-zinc-400 pt-2 border-t border-zinc-800/50">
                      <span>⏱️ <strong>{act.durationMin} min</strong></span>
                      {#if act.distanceKm > 0}
                        <span>📍 <strong>{act.distanceKm.toFixed(2)} km</strong></span>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {:else if activeTab === 'activities'}
            <div class="p-8 text-center text-zinc-400 text-xs border border-dashed border-zinc-800 rounded-3xl space-y-2">
              <Activity class="w-8 h-8 text-zinc-600 mx-auto" />
              <p>No Garmin activities synced for this date.</p>
            </div>
          {/if}
        {/if}

        <!-- Device Calendar Events Section -->
        {#if activeTab === 'all' || activeTab === 'schedule'}
          {#if day.events && day.events.length > 0}
            <div class="space-y-2.5">
              <span class="text-xs font-bold text-indigo-400 flex items-center gap-1.5">
                <CalendarIcon class="w-4 h-4" /> Device Calendar Schedule ({day.events.length})
              </span>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each day.events as ev}
                  <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 space-y-2 relative group shadow-sm">
                    <div class="flex items-start justify-between gap-2">
                      <div>
                        <div class="flex items-center gap-1.5">
                          <span class="text-[10px] font-bold uppercase tracking-wider text-indigo-400 bg-indigo-500/10 px-2 py-0.5 rounded-md border border-indigo-500/20">
                            {ev.calendarName || 'Samsung Calendar'}
                          </span>
                          {#if ev.isAllDay}
                            <span class="text-[9px] font-bold px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-300 font-mono">ALL DAY</span>
                          {/if}
                        </div>
                        <h4 class="text-sm font-bold text-zinc-100 mt-1">{ev.title}</h4>
                      </div>
                      
                      {#if !ev.isAllDay}
                        <div class="text-right">
                          <span class="text-xs font-mono font-bold text-zinc-300">
                            {formatEventTime(ev.startTime)} - {formatEventTime(ev.endTime)}
                          </span>
                        </div>
                      {/if}
                    </div>

                    {#if ev.location}
                      <p class="text-xs text-zinc-400 flex items-center gap-1">
                        <span>📍</span> <span class="truncate">{ev.location}</span>
                      </p>
                    {/if}

                    {#if ev.description}
                      <p class="text-xs text-zinc-400 italic bg-zinc-900/60 p-2 rounded-xl line-clamp-2">"{ev.description}"</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {:else if activeTab === 'schedule'}
            <div class="p-8 text-center text-zinc-400 text-xs border border-dashed border-zinc-800 rounded-3xl space-y-3">
              <CalendarIcon class="w-8 h-8 text-indigo-400 mx-auto" />
              <p>No calendar events found for this day.</p>
              <button
                onclick={async () => { await syncDeviceCalendar(); await loadCalendarMonth(); }}
                class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs cursor-pointer transition shadow-md"
              >
                Sync Device Calendar Now
              </button>
            </div>
          {/if}
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- Log Food Modal -->
{#if showFoodModal}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-5 md:p-6 w-full max-w-lg shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <Utensils class="w-5 h-5 text-amber-400" />
          <h3 class="text-base font-bold text-zinc-100">Log Nutrition / Meal</h3>
        </div>
        <button 
          onclick={() => showFoodModal = false}
          class="p-1 text-zinc-400 hover:text-zinc-100 rounded-xl hover:bg-zinc-800 transition-colors cursor-pointer"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Meal Type Selection -->
      <div class="space-y-2">
        <span class="text-xs font-bold text-zinc-300 block">Meal Type</span>
        <div class="grid grid-cols-3 sm:grid-cols-6 gap-2">
          {#each MEAL_TYPES as mt}
            <button 
              onclick={() => mealType = mt.key}
              class="p-2 rounded-xl border text-center transition-all flex flex-col items-center gap-1 cursor-pointer {mealType === mt.key ? 'bg-amber-500/20 border-amber-500 text-amber-300' : 'bg-zinc-950 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
            >
              <span class="text-sm">{mt.icon}</span>
              <span class="text-[10px] font-bold truncate">{mt.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Meal Description & Voice Input -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-bold text-zinc-300 block">Description / Voice</span>
          <button
            type="button"
            onclick={toggleFoodVoiceRecording}
            class="px-2.5 py-1 rounded-xl text-[11px] font-bold flex items-center gap-1.5 transition-all cursor-pointer border {isFoodVoiceRecording ? 'bg-rose-600 hover:bg-rose-500 text-white border-rose-400 shadow-[0_0_12px_rgba(244,63,94,0.5)] animate-pulse' : 'bg-zinc-800 hover:bg-zinc-700 text-amber-300 border-zinc-700'}"
          >
            {#if isFoodVoiceRecording}
              <Square class="w-3 h-3 fill-current" />
              <span>Stop Recording</span>
            {:else}
              <Mic class="w-3 h-3 text-amber-400" />
              <span>Record Voice</span>
            {/if}
          </button>
        </div>
        <input 
          type="text" 
          bind:value={foodDescription}
          placeholder={isFoodVoiceRecording ? 'Listening to voice message...' : 'e.g., 250ml coffee and grilled salmon bowl'}
          class="w-full bg-zinc-950 border {isFoodVoiceRecording ? 'border-rose-500/60' : 'border-zinc-800'} rounded-2xl px-3.5 py-2.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-amber-500 transition-colors"
        />
      </div>

      <!-- Calories & Macros -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div class="space-y-1">
          <span class="text-[10px] font-bold text-zinc-400 block uppercase">kcal</span>
          <input type="number" bind:value={foodCalories} class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-amber-400 focus:outline-none focus:border-amber-500" />
        </div>
        <div class="space-y-1">
          <span class="text-[10px] font-bold text-zinc-400 block uppercase">Carbs</span>
          <input type="number" bind:value={foodCarbs} class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-amber-500" />
        </div>
        <div class="space-y-1">
          <span class="text-[10px] font-bold text-zinc-400 block uppercase">Prot</span>
          <input type="number" bind:value={foodProtein} class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-amber-500" />
        </div>
        <div class="space-y-1">
          <span class="text-[10px] font-bold text-zinc-400 block uppercase">Fat</span>
          <input type="number" bind:value={foodFat} class="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-amber-500" />
        </div>
      </div>

      <!-- Water -->
      <div class="space-y-1.5">
        <span class="text-xs font-bold text-zinc-300 block">Water (ml)</span>
        <input type="number" bind:value={foodWater} placeholder="0" class="w-full bg-zinc-950 border border-zinc-800 rounded-2xl px-3.5 py-2 text-xs text-cyan-300 font-mono focus:outline-none focus:border-cyan-500" />
      </div>

      <!-- Notes -->
      <div class="space-y-1.5">
        <span class="text-xs font-bold text-zinc-300 block">Notes</span>
        <textarea bind:value={foodNotes} rows="2" class="w-full bg-zinc-950 border border-zinc-800 rounded-2xl px-3.5 py-2 text-xs text-zinc-100 focus:outline-none focus:border-amber-500"></textarea>
      </div>

      <div class="flex items-center justify-end gap-3 pt-2">
        <button 
          onclick={() => showFoodModal = false}
          class="px-4 py-2 rounded-2xl border border-zinc-800 text-zinc-400 hover:text-zinc-100 text-xs font-bold cursor-pointer"
        >
          Cancel
        </button>
        <button 
          onclick={submitFoodLog}
          class="px-5 py-2.5 rounded-2xl bg-amber-500 hover:bg-amber-400 text-zinc-950 text-xs font-black shadow-lg shadow-amber-500/20 transition-all cursor-pointer"
        >
          Save Food Entry
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Log Emotion Check-in Modal -->
{#if showEmotionModal}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-5 md:p-6 w-full max-w-lg shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center gap-2">
          <HeartHandshake class="w-5 h-5 text-cyan-400" />
          <h3 class="text-base font-bold text-zinc-100">Subjective Emotional Check-in</h3>
        </div>
        <button 
          onclick={() => showEmotionModal = false}
          class="p-1 text-zinc-400 hover:text-zinc-100 rounded-xl hover:bg-zinc-800 transition-colors cursor-pointer"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Auto-Detected Device Time Banner -->
      <div class="flex items-center justify-between px-3.5 py-2.5 rounded-2xl bg-zinc-950/80 border border-zinc-800 text-xs text-zinc-400 font-mono">
        <span class="flex items-center gap-1.5 text-zinc-300 font-sans font-medium">
          <Clock class="w-3.5 h-3.5 text-cyan-400" />
          <span>Device Time Check-in</span>
        </span>
        <span class="capitalize font-bold text-cyan-300">
          {new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} • {getDeviceTimeOfDay()}
        </span>
      </div>

      <!-- Primary Mood Picker -->
      <div class="space-y-2">
        <span class="text-xs font-bold text-zinc-300 block">Primary Subjective Mood</span>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
          {#each MOOD_OPTIONS as opt}
            <button 
              onclick={() => selectedMood = opt.key}
              class="p-2.5 rounded-2xl border text-left transition-all flex items-center gap-2 cursor-pointer {selectedMood === opt.key ? `${opt.color} ring-1 ring-cyan-500` : 'bg-zinc-950 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
            >
              <span class="text-base">{opt.icon}</span>
              <span class="text-xs font-bold">{opt.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- 1 to 10 Sliders -->
      <div class="space-y-3.5 bg-zinc-950/80 p-4 rounded-2xl border border-zinc-800/80">
        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs"><span class="font-bold text-zinc-300">Energy (1-10)</span><span class="font-black text-amber-400">{energyLevel}</span></div>
          <input type="range" min="1" max="10" bind:value={energyLevel} class="w-full accent-amber-400" />
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs"><span class="font-bold text-zinc-300">Drive (1-10)</span><span class="font-black text-cyan-400">{motivationLevel}</span></div>
          <input type="range" min="1" max="10" bind:value={motivationLevel} class="w-full accent-cyan-400" />
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs"><span class="font-bold text-zinc-300">Stress (1-10)</span><span class="font-black text-rose-400">{stressLevel}</span></div>
          <input type="range" min="1" max="10" bind:value={stressLevel} class="w-full accent-rose-400" />
        </div>
      </div>

      <!-- What's Stressing You (Custom Entry + Quick Suggestions) -->
      <div class="space-y-2.5">
        <div class="flex items-center justify-between">
          <label for="customStressorInput" class="text-xs font-bold text-zinc-300 block">What is stressing you?</label>
          <span class="text-[10px] text-zinc-500 font-medium">Type custom or select tags below</span>
        </div>

        <!-- Custom Stressor Input Field -->
        <div class="flex items-center gap-2">
          <input
            id="customStressorInput"
            type="text"
            bind:value={customStressorInput}
            onkeydown={handleStressorKeyDown}
            placeholder="Type anything (e.g. Flight delay, Knee ache, Client meeting)..."
            class="flex-1 bg-zinc-950 border border-zinc-800 focus:border-cyan-500 rounded-xl px-3.5 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none transition-colors"
          />
          <button
            type="button"
            onclick={addCustomStressor}
            disabled={!customStressorInput.trim()}
            class="px-3.5 py-2 rounded-xl bg-zinc-800 hover:bg-zinc-700 disabled:opacity-40 text-cyan-300 text-xs font-bold border border-zinc-700 transition-all cursor-pointer flex items-center gap-1 shrink-0"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>Add</span>
          </button>
        </div>

        <!-- Active Selected Stressors Tag List -->
        {#if selectedStressors.length > 0}
          <div class="space-y-1 bg-zinc-950/60 p-2.5 rounded-2xl border border-zinc-800/80">
            <span class="text-[10px] uppercase font-bold text-zinc-400 block tracking-wider">Active Stressors:</span>
            <div class="flex flex-wrap gap-1.5 pt-0.5">
              {#each selectedStressors as st}
                <span class="inline-flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-xl bg-rose-500/20 text-rose-300 border border-rose-500/40 font-semibold shadow-sm">
                  <span>{st}</span>
                  <button
                    type="button"
                    onclick={() => removeStressor(st)}
                    class="hover:text-white rounded-full p-0.5 text-rose-400 hover:bg-rose-500/30 transition cursor-pointer"
                    aria-label={`Remove ${st}`}
                  >
                    <X class="w-3 h-3" />
                  </button>
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Quick Suggestion Tags -->
        <div class="space-y-1">
          <span class="text-[11px] font-semibold text-zinc-400 block">Quick Suggestions:</span>
          <div class="flex flex-wrap gap-1.5">
            {#each COMMON_STRESSORS as st}
              {@const isSelected = selectedStressors.includes(st)}
              <button 
                type="button"
                onclick={() => toggleStressor(st)}
                class="text-xs px-2.5 py-1 rounded-xl border transition-all cursor-pointer {isSelected ? 'bg-rose-500/20 border-rose-500 text-rose-300' : 'bg-zinc-950 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
              >
                {isSelected ? '✓ ' : '+ '}{st}
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Additional Thoughts & Reflections Notes -->
      <div class="space-y-1.5">
        <label for="emotionNotes" class="text-xs font-bold text-zinc-300 block">Check-in Notes & Journal Thoughts (Optional)</label>
        <textarea
          id="emotionNotes"
          bind:value={emotionNotes}
          rows="2"
          placeholder="Describe how you're feeling, triggers, or what you plan to do..."
          class="w-full bg-zinc-950 border border-zinc-800 rounded-2xl px-3.5 py-2 text-xs text-zinc-100 focus:outline-none focus:border-cyan-500"
        ></textarea>
      </div>

      <div class="flex items-center justify-end gap-3 pt-2">
        <button 
          onclick={() => showEmotionModal = false}
          class="px-4 py-2 rounded-2xl border border-zinc-800 text-zinc-400 hover:text-zinc-100 text-xs font-bold cursor-pointer"
        >
          Cancel
        </button>
        <button 
          onclick={submitEmotionLog}
          class="px-5 py-2 rounded-2xl bg-cyan-500 text-zinc-950 font-black text-xs hover:bg-cyan-400 transition-colors shadow-lg cursor-pointer"
        >
          Record Emotional Check-in
        </button>
      </div>
    </div>
  </div>
{/if}
